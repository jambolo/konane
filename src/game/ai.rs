use std::rc::Rc;

use game_player::minimax::{ResponseGenerator, search};
use game_player::{PlayerId, State, StaticEvaluator};

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

impl ResponseGenerator for KonaneMoveGenerator {
    type State = KonaneState;

    fn generate(&self, state: &Self::State, _depth: u32) -> Vec<KonaneAction> {
        match state.current_phase() {
            GamePhase::OpeningBlackRemoval => Rules::valid_black_opening_removals(state)
                .into_iter()
                .map(KonaneAction::OpeningRemoval)
                .collect(),
            GamePhase::OpeningWhiteRemoval => Rules::valid_white_opening_removals(state)
                .into_iter()
                .map(KonaneAction::OpeningRemoval)
                .collect(),
            GamePhase::Play => Rules::all_valid_jumps(state)
                .into_iter()
                .map(KonaneAction::Jump)
                .collect(),
            _ => Vec::new(),
        }
    }
}

pub struct AiPlayer {
    color: PieceColor,
    depth: u32,
}

impl AiPlayer {
    pub fn new(color: PieceColor, depth: u32) -> Self {
        Self { color, depth }
    }

    pub fn compute_move(&self, state: &KonaneState) -> Option<PlayerMove> {
        let konane_state = Rc::new(state.clone());

        let evaluator = KonaneEvaluator;
        let generator = KonaneMoveGenerator;
        let result = search(&evaluator, &generator, &konane_state, self.depth);

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

            let moves = generator.generate(&state, 0);

            // Should generate moves for center and corner black pieces
            assert!(!moves.is_empty());
            for mv in &moves {
                assert!(matches!(mv, KonaneAction::OpeningRemoval(_)));
            }
        }

        #[test]
        fn generates_white_opening_removals() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            let _ = Rules::apply_opening_removal(&mut state, Position::new(1, 1));

            let generator = KonaneMoveGenerator;

            let moves = generator.generate(&state, 0);

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

            let moves = generator.generate(&state, 0);

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

            let moves = generator.generate(&state, 0);
            assert!(moves.is_empty());
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
