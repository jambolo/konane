use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use game_player::minimax::{search, ResponseGenerator};
use game_player::{PlayerId, State, StaticEvaluator, TranspositionTable};

use crate::game::player::{Player, PlayerInput, PlayerMove};
use crate::game::rules::{Jump, Rules};
use crate::game::state::{Cell, GamePhase, GameState, PieceColor, Position};

#[derive(Debug, Clone)]
pub enum KonaneAction {
    OpeningRemoval(Position),
    Jump(Jump),
}

pub struct KonaneState {
    pub inner: GameState,
    pub last_action: Option<KonaneAction>,
}

impl State for KonaneState {
    type Action = KonaneAction;

    fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        let size = self.inner.board.size();
        for row in 0..size {
            for col in 0..size {
                let pos = Position::new(row, col);
                let cell_val = match self.inner.board.get(pos) {
                    Some(Cell::Empty) => 0u8,
                    Some(Cell::Occupied(PieceColor::Black)) => 1u8,
                    Some(Cell::Occupied(PieceColor::White)) => 2u8,
                    None => 0u8,
                };
                cell_val.hash(&mut hasher);
            }
        }
        self.inner.current_player.hash(&mut hasher);
        std::mem::discriminant(&self.inner.phase).hash(&mut hasher);
        hasher.finish()
    }

    fn whose_turn(&self) -> u8 {
        match self.inner.current_player {
            PieceColor::Black => PlayerId::ALICE as u8,
            PieceColor::White => PlayerId::BOB as u8,
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self.inner.phase, GamePhase::GameOver { .. })
    }

    fn apply(&self, action: &KonaneAction) -> Self {
        let mut new_state = self.inner.clone();
        match action {
            KonaneAction::OpeningRemoval(pos) => {
                let _ = Rules::apply_opening_removal(&mut new_state, *pos);
            }
            KonaneAction::Jump(jump) => {
                Rules::apply_jump(&mut new_state, jump);
            }
        }
        KonaneState {
            inner: new_state,
            last_action: Some(action.clone()),
        }
    }
}

pub struct KonaneEvaluator;

impl StaticEvaluator<KonaneState> for KonaneEvaluator {
    fn evaluate(&self, state: &KonaneState) -> f32 {
        if let GamePhase::GameOver { winner } = state.inner.phase {
            return if winner == PieceColor::Black {
                self.alice_wins_value()
            } else {
                self.bob_wins_value()
            };
        }

        // Mobility heuristic: count valid moves for each player
        let black_mobility = count_mobility_for(&state.inner, PieceColor::Black);
        if state.inner.current_player == PieceColor::Black && black_mobility == 0 {
            return self.bob_wins_value();
        }
        let white_mobility = count_mobility_for(&state.inner, PieceColor::White);
        if state.inner.current_player == PieceColor::White && white_mobility == 0 {
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

fn count_mobility_for(state: &GameState, color: PieceColor) -> i32 {
    let mut temp_state = state.clone();
    temp_state.current_player = color;

    match temp_state.phase {
        GamePhase::Play | GamePhase::GameOver { .. } => {
            Rules::all_valid_jumps(&temp_state).len() as i32
        }
        _ => 0,
    }
}

pub struct KonaneMoveGenerator;

impl ResponseGenerator for KonaneMoveGenerator {
    type State = KonaneState;

    fn generate(&self, state: &Rc<Self::State>, _depth: i32) -> Vec<Box<Self::State>> {
        let inner = &state.inner;

        match inner.phase {
            GamePhase::OpeningBlackRemoval => Rules::valid_black_opening_removals(inner)
                .into_iter()
                .map(|pos| {
                    let action = KonaneAction::OpeningRemoval(pos);
                    Box::new(state.apply(&action))
                })
                .collect(),
            GamePhase::OpeningWhiteRemoval => Rules::valid_white_opening_removals(inner)
                .into_iter()
                .map(|pos| {
                    let action = KonaneAction::OpeningRemoval(pos);
                    Box::new(state.apply(&action))
                })
                .collect(),
            GamePhase::Play => Rules::all_valid_jumps(inner)
                .into_iter()
                .map(|jump| {
                    let action = KonaneAction::Jump(jump);
                    Box::new(state.apply(&action))
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}

pub struct AiPlayer {
    color: PieceColor,
    depth: i32,
}

impl AiPlayer {
    pub fn new(color: PieceColor, depth: i32) -> Self {
        Self { color, depth }
    }

    pub fn compute_move(&self, state: &GameState) -> Option<PlayerMove> {
        let konane_state = Rc::new(KonaneState {
            inner: state.clone(),
            last_action: None,
        });

        let evaluator = KonaneEvaluator;
        let generator = KonaneMoveGenerator;
        let tt = Rc::new(RefCell::new(TranspositionTable::new(100_000, 100)));

        let result = search(&tt, &evaluator, &generator, &konane_state, self.depth);

        result
            .and_then(|best_state| best_state.last_action.clone())
            .map(|action| match action {
                KonaneAction::OpeningRemoval(pos) => PlayerMove::OpeningRemoval(pos),
                KonaneAction::Jump(jump) => PlayerMove::Jump(jump),
            })
    }
}

impl Player for AiPlayer {
    fn color(&self) -> PieceColor {
        self.color
    }

    fn request_move(&mut self, state: &GameState) -> Option<PlayerMove> {
        self.compute_move(state)
    }

    fn receive_input(&mut self, _input: PlayerInput) {
        // AI ignores UI input
    }

    fn is_ready(&self) -> bool {
        true
    }
}
