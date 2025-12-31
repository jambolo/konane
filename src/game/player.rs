use crate::game::rules::Jump;
use crate::game::state::*;

// Represents a move that a player can make
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PlayerMove {
    OpeningRemoval(Position),
    Jump(Jump),
}

// Trait for player implementations
// This allows for different player types (human, AI, network, etc.)
#[allow(dead_code)]
pub trait Player {
    fn color(&self) -> PieceColor;

    // Called when it's this player's turn
    // Human players return None and wait for UI input
    // AI players could compute and return a move directly
    fn request_move(&mut self, state: &GameState) -> Option<PlayerMove>;

    // For human players, this is called when the UI receives input
    fn receive_input(&mut self, input: PlayerInput);

    // Check if the player is ready to provide a move
    fn is_ready(&self) -> bool;
}

// Input from UI for human players
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PlayerInput {
    PositionSelected(Position),
    JumpSelected(Jump),
    Cancel,
}

// Human player implementation
#[allow(dead_code)]
pub struct HumanPlayer {
    color: PieceColor,
    pending_move: Option<PlayerMove>,
}

impl HumanPlayer {
    pub fn _new(color: PieceColor) -> Self {
        Self {
            color,
            pending_move: None,
        }
    }
}

impl Player for HumanPlayer {
    fn color(&self) -> PieceColor {
        self.color
    }

    fn request_move(&mut self, _state: &GameState) -> Option<PlayerMove> {
        self.pending_move.take()
    }

    fn receive_input(&mut self, input: PlayerInput) {
        match input {
            PlayerInput::PositionSelected(pos) => {
                self.pending_move = Some(PlayerMove::OpeningRemoval(pos));
            }
            PlayerInput::JumpSelected(jump) => {
                self.pending_move = Some(PlayerMove::Jump(jump));
            }
            PlayerInput::Cancel => {
                self.pending_move = None;
            }
        }
    }

    fn is_ready(&self) -> bool {
        self.pending_move.is_some()
    }
}
