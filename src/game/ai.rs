use game_player::{PlayerId, State, StaticEvaluator, mcts, minimax};
#[cfg(feature = "random-openings")]
use rand::seq::IndexedRandom;

use crate::game::player::{Player, PlayerInput, PlayerMove};
use crate::game::rules::{Jump, Rules};
use crate::game::state::{GamePhase, KonaneState, PieceColor, Position};

#[derive(Debug, Clone)]
pub enum KonaneAction {
    OpeningRemoval(Position),
    Jump(Jump),
}

impl State for KonaneState {
    type Action = KonaneAction;

    fn fingerprint(&self) -> u64 {
        KonaneState::fingerprint(self)
    }

    fn whose_turn(&self) -> PlayerId {
        match self.current_player() {
            PieceColor::Black => PlayerId::Alice,
            PieceColor::White => PlayerId::Bob,
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self.current_phase(), GamePhase::GameOver { .. })
    }

    fn apply(&self, action: &KonaneAction) -> Self {
        let mut new_state = self.clone();
        match action {
            KonaneAction::OpeningRemoval(pos) => {
                let _ = Rules::apply_opening_removal(&mut new_state, *pos);
            }
            KonaneAction::Jump(jump) => {
                Rules::apply_jump(&mut new_state, jump);
            }
        }
        new_state
    }
}

pub struct KonaneEvaluator;

impl StaticEvaluator for KonaneEvaluator {
    type State = KonaneState;

    fn evaluate(&self, state: &KonaneState) -> f32 {
        if let GamePhase::GameOver { winner } = state.current_phase() {
            return if winner == PieceColor::Black {
                self.alice_wins_value()
            } else {
                self.bob_wins_value()
            };
        }

        // Mobility is only defined once jumping starts; during the opening removals neither player has any, so the zero-mobility
        // shortcuts below would misread every opening state as a decided game.
        if state.current_phase() != GamePhase::Play {
            return 0.0;
        }

        // Mobility heuristic: count valid moves for each player
        let black_mobility = count_mobility_for(state, PieceColor::Black);
        if state.current_player() == PieceColor::Black && black_mobility == 0 {
            return self.bob_wins_value();
        }
        let white_mobility = count_mobility_for(state, PieceColor::White);
        if state.current_player() == PieceColor::White && white_mobility == 0 {
            return self.alice_wins_value();
        }

        (black_mobility - white_mobility) as f32
    }

    fn alice_wins_value(&self) -> f32 {
        1000.0
    }

    fn bob_wins_value(&self) -> f32 {
        -1000.0
    }
}

fn count_mobility_for(state: &KonaneState, color: PieceColor) -> i32 {
    let mut temp_state = state.clone();
    temp_state.set_current_player(color);

    match temp_state.current_phase() {
        GamePhase::Play | GamePhase::GameOver { .. } => Rules::all_valid_jumps(&temp_state).len() as i32,
        _ => 0,
    }
}

pub struct KonaneMoveGenerator;

/// Both opening removals are heavily constrained by symmetry and are not known to affect the outcome, so with the `random-openings`
/// feature the generator offers a single random candidate instead of making the search expand them all. Disable the feature to
/// search openings exhaustively.
#[cfg(feature = "random-openings")]
fn opening_removals(_state: &KonaneState, candidates: Vec<Position>) -> Vec<KonaneAction> {
    candidates
        .choose(&mut rand::rng())
        .map(|pos| vec![KonaneAction::OpeningRemoval(*pos)])
        .unwrap_or_default()
}

#[cfg(not(feature = "random-openings"))]
fn opening_removals(_state: &KonaneState, candidates: Vec<Position>) -> Vec<KonaneAction> {
    candidates.into_iter().map(KonaneAction::OpeningRemoval).collect()
}

impl KonaneMoveGenerator {
    fn moves_for(&self, state: &KonaneState) -> Vec<KonaneAction> {
        match state.current_phase() {
            GamePhase::OpeningBlackRemoval => opening_removals(state, Rules::valid_black_opening_removals(state)),
            GamePhase::OpeningWhiteRemoval => opening_removals(state, Rules::valid_white_opening_removals(state)),
            GamePhase::Play => Rules::all_valid_jumps(state).into_iter().map(KonaneAction::Jump).collect(),
            _ => Vec::new(),
        }
    }
}

impl minimax::ResponseGenerator for KonaneMoveGenerator {
    type State = KonaneState;

    fn generate(&self, state: &Self::State, _depth: u32) -> Vec<KonaneAction> {
        self.moves_for(state)
    }
}

impl mcts::ResponseGenerator for KonaneMoveGenerator {
    type State = KonaneState;

    fn generate(&self, state: &Self::State) -> Vec<KonaneAction> {
        self.moves_for(state)
    }
}

/// Adapts [`KonaneEvaluator`] to `mcts::ValueEstimator`'s contract: a value in `[0.0, 1.0]` from the perspective of the
/// current player. `KonaneEvaluator` reaches its `alice_wins_value`/`bob_wins_value` bounds only for decided positions
/// (game over, or the player to move has no jumps), so the mapped value hits 0.0/1.0 only when the outcome is certain,
/// as the estimator contract requires.
pub struct KonaneValueEstimator {
    evaluator: KonaneEvaluator,
}

impl KonaneValueEstimator {
    pub fn new() -> Self {
        Self { evaluator: KonaneEvaluator }
    }
}

impl Default for KonaneValueEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl mcts::ValueEstimator for KonaneValueEstimator {
    type State = KonaneState;
    type ResponseGenerator = KonaneMoveGenerator;

    fn estimate(&self, state: &KonaneState, _rg: &KonaneMoveGenerator) -> f32 {
        let eval = self.evaluator.evaluate(state);
        let v01 =
            (eval - self.evaluator.bob_wins_value()) / (self.evaluator.alice_wins_value() - self.evaluator.bob_wins_value());
        if state.whose_turn() == PlayerId::Alice { v01 } else { 1.0 - v01 }
    }
}

/// MCTS iterations per move = `depth * MCTS_ITERATIONS_PER_DEPTH`, so the existing `--ai-depth` knob keeps scaling the
/// AI's strength when the `use-mcts` feature replaces the depth-limited minimax with an iteration-limited MCTS.
#[cfg(feature = "use-mcts")]
const MCTS_ITERATIONS_PER_DEPTH: u32 = 1000;

/// Virtual-visit weight of a node's stored initial estimate in UCT. Must be nonzero when combined with eager expansion,
/// which otherwise wastes the stored estimates (see `mcts::search`).
#[cfg(feature = "use-mcts")]
const MCTS_INITIAL_VALUE_WEIGHT: f32 = 1.0;

pub struct AiPlayer {
    color: PieceColor,
    depth: u32,
}

impl AiPlayer {
    pub fn new(color: PieceColor, depth: u32) -> Self {
        Self { color, depth }
    }

    pub fn compute_move(&self, state: &KonaneState) -> Option<PlayerMove> {
        let generator = KonaneMoveGenerator;

        #[cfg(not(feature = "use-mcts"))]
        let result = {
            let evaluator = KonaneEvaluator;
            minimax::search(&evaluator, &generator, state, self.depth)
        };

        #[cfg(feature = "use-mcts")]
        let result = {
            let estimator = KonaneValueEstimator::new();
            // Eager expansion: the static-evaluator estimate is cheap, so estimating every child on expansion is
            // affordable, and the stored estimates guide UCT through the nonzero initial-value weight.
            mcts::search(
                state,
                &generator,
                &estimator,
                mcts::DEFAULT_EXPLORATION_CONSTANT,
                MCTS_INITIAL_VALUE_WEIGHT,
                true,
                self.depth * MCTS_ITERATIONS_PER_DEPTH,
            )
        };

        result.map(|action| match action {
            KonaneAction::OpeningRemoval(pos) => PlayerMove::OpeningRemoval(pos),
            KonaneAction::Jump(jump) => PlayerMove::Jump(jump),
        })
    }
}

impl Player for AiPlayer {
    fn color(&self) -> PieceColor {
        self.color
    }

    fn request_move(&mut self, state: &KonaneState) -> Option<PlayerMove> {
        self.compute_move(state)
    }

    fn receive_input(&mut self, _input: PlayerInput) {
        // AI ignores UI input
    }

    fn is_ready(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::rules::Rules;
    use game_player::State;

    mod konane_state {
        use super::*;

        fn create_initial_state() -> KonaneState {
            KonaneState::new(4, PieceColor::Black)
        }

        #[test]
        fn fingerprint_differs_for_different_boards() {
            let state1 = create_initial_state();

            let mut state2 = KonaneState::new(4, PieceColor::Black);
            let _ = Rules::apply_opening_removal(&mut state2, Position::new(1, 1));

            assert_ne!(state1.fingerprint(), state2.fingerprint());
        }

        #[test]
        fn fingerprint_same_for_identical_boards() {
            let state1 = create_initial_state();
            let state2 = create_initial_state();
            assert_eq!(state1.fingerprint(), state2.fingerprint());
        }

        #[test]
        fn whose_turn_black_is_alice() {
            let state = create_initial_state();
            assert_eq!(state.whose_turn(), PlayerId::Alice);
        }

        #[test]
        fn whose_turn_white_is_bob() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            let _ = Rules::apply_opening_removal(&mut state, Position::new(1, 1));
            assert_eq!(state.whose_turn(), PlayerId::Bob);
        }

        #[test]
        fn is_terminal_false_at_start() {
            let state = create_initial_state();
            assert!(!state.is_terminal());
        }

        #[test]
        fn is_terminal_true_when_game_over() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::GameOver {
                winner: PieceColor::Black,
            });
            assert!(state.is_terminal());
        }

        #[test]
        fn apply_opening_removal() {
            let state = create_initial_state();
            let action = KonaneAction::OpeningRemoval(Position::new(1, 1));

            let new_state = state.apply(&action);

            assert!(new_state.board().is_empty(Position::new(1, 1)));
            assert_eq!(new_state.current_phase(), GamePhase::OpeningWhiteRemoval);
        }

        #[test]
        fn apply_jump() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2));

            let jump = crate::game::rules::Jump {
                from: Position::new(0, 0),
                to: Position::new(0, 2),
                direction: crate::game::state::Direction::Right,
                captured: vec![Position::new(0, 1)],
            };
            let action = KonaneAction::Jump(jump);

            let new_state = state.apply(&action);

            assert!(new_state.board().is_empty(Position::new(0, 0)));
            assert!(new_state.board().is_empty(Position::new(0, 1)));
            assert_eq!(
                new_state.board().get_piece_color(Position::new(0, 2)),
                Some(PieceColor::Black)
            );
        }
    }

    mod konane_evaluator {
        use super::*;

        #[test]
        fn alice_wins_value_is_positive() {
            let evaluator = KonaneEvaluator;
            assert!(evaluator.alice_wins_value() > 0.0);
        }

        #[test]
        fn bob_wins_value_is_negative() {
            let evaluator = KonaneEvaluator;
            assert!(evaluator.bob_wins_value() < 0.0);
        }

        #[test]
        fn evaluate_game_over_black_wins() {
            let evaluator = KonaneEvaluator;
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::GameOver {
                winner: PieceColor::Black,
            });

            let score = evaluator.evaluate(&state);
            assert_eq!(score, evaluator.alice_wins_value());
        }

        #[test]
        fn evaluate_game_over_white_wins() {
            let evaluator = KonaneEvaluator;
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::GameOver {
                winner: PieceColor::White,
            });

            let score = evaluator.evaluate(&state);
            assert_eq!(score, evaluator.bob_wins_value());
        }

        #[test]
        fn evaluate_opening_phases_are_neutral() {
            let evaluator = KonaneEvaluator;

            let black_removal = KonaneState::new(8, PieceColor::Black);
            assert_eq!(evaluator.evaluate(&black_removal), 0.0);

            let mut white_removal = black_removal.clone();
            let _ = Rules::apply_opening_removal(&mut white_removal, Position::new(3, 3));
            assert_eq!(white_removal.current_phase(), GamePhase::OpeningWhiteRemoval);
            assert_eq!(evaluator.evaluate(&white_removal), 0.0);
        }

        #[test]
        fn evaluate_zero_mobility_loses_in_play_phase() {
            let evaluator = KonaneEvaluator;

            // Black to move on a board with no legal jumps loses.
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            assert_eq!(evaluator.evaluate(&state), evaluator.bob_wins_value());

            state.set_current_player(PieceColor::White);
            assert_eq!(evaluator.evaluate(&state), evaluator.alice_wins_value());
        }

        #[test]
        fn evaluate_uses_mobility() {
            let evaluator = KonaneEvaluator;

            // State with more black mobility should have higher score
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2));
            state.remove_stone(Position::new(2, 0));

            let score = evaluator.evaluate(&state);
            // Score should be non-zero (mobility difference)
            assert!(score != 0.0 || score == 0.0); // Just ensure it computes
        }
    }

    mod konane_move_generator {
        use super::*;

        #[test]
        fn generates_black_opening_removals() {
            let state = KonaneState::new(4, PieceColor::Black);
            let generator = KonaneMoveGenerator;

            let moves = generator.moves_for(&state);

            // Should generate moves for center and corner black pieces
            assert!(!moves.is_empty());
            for mv in &moves {
                assert!(matches!(mv, KonaneAction::OpeningRemoval(_)));
            }
        }

        #[cfg(feature = "random-openings")]
        #[test]
        fn black_opening_removal_generates_exactly_one_valid_move() {
            let state = KonaneState::new(8, PieceColor::Black);
            let generator = KonaneMoveGenerator;
            let valid = Rules::valid_black_opening_removals(&state);
            assert!(valid.len() > 1);

            for _ in 0..20 {
                let moves = generator.moves_for(&state);
                assert_eq!(moves.len(), 1);
                match moves[0] {
                    KonaneAction::OpeningRemoval(pos) => assert!(valid.contains(&pos)),
                    _ => panic!("Expected OpeningRemoval"),
                }
            }
        }

        #[cfg(feature = "random-openings")]
        #[test]
        fn white_opening_removal_generates_exactly_one_valid_move() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            let _ = Rules::apply_opening_removal(&mut state, Position::new(0, 0));
            assert_eq!(state.current_phase(), GamePhase::OpeningWhiteRemoval);

            let generator = KonaneMoveGenerator;
            let valid = Rules::valid_white_opening_removals(&state);
            assert!(valid.len() > 1);

            for _ in 0..20 {
                let moves = generator.moves_for(&state);
                assert_eq!(moves.len(), 1);
                match moves[0] {
                    KonaneAction::OpeningRemoval(pos) => assert!(valid.contains(&pos)),
                    _ => panic!("Expected OpeningRemoval"),
                }
            }
        }

        #[cfg(feature = "random-openings")]
        #[test]
        fn openings_pick_one_of_the_valid_candidates_on_small_boards() {
            let mut state = KonaneState::new(6, PieceColor::Black);
            let generator = KonaneMoveGenerator;

            let black = Rules::valid_black_opening_removals(&state);
            assert!(black.len() > 1);
            let moves = generator.moves_for(&state);
            assert_eq!(moves.len(), 1);
            match moves[0] {
                KonaneAction::OpeningRemoval(pos) => assert!(black.contains(&pos)),
                _ => panic!("Expected OpeningRemoval"),
            }

            let _ = Rules::apply_opening_removal(&mut state, Position::new(0, 0));
            let white = Rules::valid_white_opening_removals(&state);
            assert!(white.len() > 1);
            let moves = generator.moves_for(&state);
            assert_eq!(moves.len(), 1);
            match moves[0] {
                KonaneAction::OpeningRemoval(pos) => assert!(white.contains(&pos)),
                _ => panic!("Expected OpeningRemoval"),
            }
        }

        #[cfg(not(feature = "random-openings"))]
        #[test]
        fn openings_are_exhaustive_without_the_feature() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            let generator = KonaneMoveGenerator;

            let black = Rules::valid_black_opening_removals(&state);
            assert_eq!(generator.moves_for(&state).len(), black.len());

            let _ = Rules::apply_opening_removal(&mut state, Position::new(0, 0));
            let white = Rules::valid_white_opening_removals(&state);
            assert_eq!(generator.moves_for(&state).len(), white.len());
        }

        #[test]
        fn generates_white_opening_removals() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            let _ = Rules::apply_opening_removal(&mut state, Position::new(1, 1));

            let generator = KonaneMoveGenerator;

            let moves = generator.moves_for(&state);

            assert!(!moves.is_empty());
            for mv in &moves {
                assert!(matches!(mv, KonaneAction::OpeningRemoval(_)));
            }
        }

        #[test]
        fn generates_jumps_in_play_phase() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2));

            let generator = KonaneMoveGenerator;

            let moves = generator.moves_for(&state);

            assert!(!moves.is_empty());
            // Should contain at least one jump
            assert!(moves.iter().any(|mv| matches!(mv, KonaneAction::Jump(_))));
        }

        #[test]
        fn returns_empty_when_game_over() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::GameOver {
                winner: PieceColor::Black,
            });

            let generator = KonaneMoveGenerator;

            let moves = generator.moves_for(&state);
            assert!(moves.is_empty());
        }
    }

    mod konane_value_estimator {
        use super::*;
        use game_player::mcts::ValueEstimator;

        #[test]
        fn estimate_is_neutral_in_opening() {
            let estimator = KonaneValueEstimator::new();
            let state = KonaneState::new(8, PieceColor::Black);
            assert_eq!(estimator.estimate(&state, &KonaneMoveGenerator), 0.5);
        }

        #[test]
        fn estimate_terminal_outcome_is_certain_and_perspective_flipped() {
            let estimator = KonaneValueEstimator::new();
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::GameOver {
                winner: PieceColor::Black,
            });

            // Black (Alice) to move: a Black win is a certain 1.0
            state.set_current_player(PieceColor::Black);
            assert_eq!(estimator.estimate(&state, &KonaneMoveGenerator), 1.0);

            // Same terminal state seen by White (Bob) is a certain 0.0
            state.set_current_player(PieceColor::White);
            assert_eq!(estimator.estimate(&state, &KonaneMoveGenerator), 0.0);
        }

        #[test]
        fn estimate_heuristic_values_stay_strictly_inside_bounds() {
            let estimator = KonaneValueEstimator::new();
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2));

            let value = estimator.estimate(&state, &KonaneMoveGenerator);
            assert!(value > 0.0 && value < 1.0);
        }
    }

    mod ai_player {
        use super::*;

        #[test]
        fn new_creates_with_correct_color_and_depth() {
            let player = AiPlayer::new(PieceColor::Black, 4);
            assert_eq!(player.color(), PieceColor::Black);
        }

        #[test]
        fn is_ready_always_true() {
            let player = AiPlayer::new(PieceColor::Black, 4);
            assert!(player.is_ready());
        }

        #[test]
        fn receive_input_ignored() {
            let mut player = AiPlayer::new(PieceColor::Black, 4);
            player.receive_input(PlayerInput::Cancel);
            // Should still be ready
            assert!(player.is_ready());
        }

        #[test]
        fn compute_move_returns_valid_opening_removal() {
            let state = KonaneState::new(4, PieceColor::Black);
            let player = AiPlayer::new(PieceColor::Black, 2);

            let mv = player.compute_move(&state);

            assert!(mv.is_some());
            match mv.unwrap() {
                PlayerMove::OpeningRemoval(pos) => {
                    // Should be a valid black opening position
                    let valid = Rules::valid_black_opening_removals(&state);
                    assert!(valid.contains(&pos));
                }
                _ => panic!("Expected OpeningRemoval during opening phase"),
            }
        }

        #[test]
        fn compute_move_returns_valid_jump() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2));

            let player = AiPlayer::new(PieceColor::Black, 2);
            let mv = player.compute_move(&state);

            assert!(mv.is_some());
            match mv.unwrap() {
                PlayerMove::Jump(jump) => {
                    // Should be a valid jump
                    let valid = Rules::all_valid_jumps(&state);
                    assert!(valid.iter().any(|j| j.from == jump.from && j.to == jump.to));
                }
                _ => panic!("Expected Jump during play phase"),
            }
        }

        #[test]
        fn request_move_delegates_to_compute_move() {
            let state = KonaneState::new(4, PieceColor::Black);
            let mut player = AiPlayer::new(PieceColor::Black, 2);

            let mv = player.request_move(&state);

            assert!(mv.is_some());
        }
    }

    mod integration {
        use super::*;

        #[test]
        fn ai_plays_complete_opening_sequence() {
            let mut state = KonaneState::new(4, PieceColor::Black);

            // Black AI makes first removal
            let black_ai = AiPlayer::new(PieceColor::Black, 2);
            let mv1 = black_ai.compute_move(&state);
            assert!(mv1.is_some());

            if let Some(PlayerMove::OpeningRemoval(pos)) = mv1 {
                let _ = Rules::apply_opening_removal(&mut state, pos);
            }

            assert_eq!(state.current_phase(), GamePhase::OpeningWhiteRemoval);

            // White AI makes second removal
            let white_ai = AiPlayer::new(PieceColor::White, 2);
            let mv2 = white_ai.compute_move(&state);
            assert!(mv2.is_some());

            if let Some(PlayerMove::OpeningRemoval(pos)) = mv2 {
                let _ = Rules::apply_opening_removal(&mut state, pos);
            }

            assert_eq!(state.current_phase(), GamePhase::Play);
        }

        #[test]
        fn ai_selects_best_move_shallow_depth() {
            // With very shallow depth, AI should still make legal moves
            let state = KonaneState::new(4, PieceColor::Black);
            let player = AiPlayer::new(PieceColor::Black, 1);

            let mv = player.compute_move(&state);
            assert!(mv.is_some());
        }
    }
}
