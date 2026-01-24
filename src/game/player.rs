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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::state::Direction;

    mod human_player {
        use super::*;

        fn create_player() -> HumanPlayer {
            HumanPlayer::_new(PieceColor::Black)
        }

        #[test]
        fn new_creates_with_color() {
            let player = HumanPlayer::_new(PieceColor::White);
            assert_eq!(player.color(), PieceColor::White);
        }

        #[test]
        fn initially_not_ready() {
            let player = create_player();
            assert!(!player.is_ready());
        }

        #[test]
        fn position_selected_makes_ready() {
            let mut player = create_player();
            player.receive_input(PlayerInput::PositionSelected(Position::new(0, 0)));
            assert!(player.is_ready());
        }

        #[test]
        fn jump_selected_makes_ready() {
            let mut player = create_player();
            let jump = Jump {
                from: Position::new(0, 0),
                to: Position::new(0, 2),
                direction: Direction::Right,
                captured: vec![Position::new(0, 1)],
            };
            player.receive_input(PlayerInput::JumpSelected(jump));
            assert!(player.is_ready());
        }

        #[test]
        fn cancel_clears_pending_move() {
            let mut player = create_player();
            player.receive_input(PlayerInput::PositionSelected(Position::new(0, 0)));
            assert!(player.is_ready());

            player.receive_input(PlayerInput::Cancel);
            assert!(!player.is_ready());
        }

        #[test]
        fn request_move_returns_and_clears() {
            let mut player = create_player();
            let state = GameState::new(8, PieceColor::Black);

            player.receive_input(PlayerInput::PositionSelected(Position::new(3, 3)));
            let mv = player.request_move(&state);

            assert!(mv.is_some());
            match mv.unwrap() {
                PlayerMove::OpeningRemoval(pos) => {
                    assert_eq!(pos, Position::new(3, 3));
                }
                _ => panic!("Expected OpeningRemoval"),
            }

            // After request_move, player is no longer ready
            assert!(!player.is_ready());
        }

        #[test]
        fn request_move_returns_none_when_not_ready() {
            let mut player = create_player();
            let state = GameState::new(8, PieceColor::Black);

            let mv = player.request_move(&state);
            assert!(mv.is_none());
        }

        #[test]
        fn request_move_returns_jump() {
            let mut player = create_player();
            let state = GameState::new(8, PieceColor::Black);

            let jump = Jump {
                from: Position::new(0, 0),
                to: Position::new(0, 2),
                direction: Direction::Right,
                captured: vec![Position::new(0, 1)],
            };
            player.receive_input(PlayerInput::JumpSelected(jump.clone()));

            let mv = player.request_move(&state);
            assert!(mv.is_some());
            match mv.unwrap() {
                PlayerMove::Jump(j) => {
                    assert_eq!(j.from, Position::new(0, 0));
                    assert_eq!(j.to, Position::new(0, 2));
                }
                _ => panic!("Expected Jump"),
            }
        }

        #[test]
        fn color_returns_assigned_color() {
            let black_player = HumanPlayer::_new(PieceColor::Black);
            let white_player = HumanPlayer::_new(PieceColor::White);

            assert_eq!(black_player.color(), PieceColor::Black);
            assert_eq!(white_player.color(), PieceColor::White);
        }
    }

    mod player_move {
        use super::*;

        #[test]
        fn opening_removal_variant() {
            let mv = PlayerMove::OpeningRemoval(Position::new(1, 1));
            match mv {
                PlayerMove::OpeningRemoval(pos) => {
                    assert_eq!(pos, Position::new(1, 1));
                }
                _ => panic!("Expected OpeningRemoval"),
            }
        }

        #[test]
        fn jump_variant() {
            let jump = Jump {
                from: Position::new(0, 0),
                to: Position::new(0, 2),
                direction: Direction::Right,
                captured: vec![Position::new(0, 1)],
            };
            let mv = PlayerMove::Jump(jump);
            match mv {
                PlayerMove::Jump(j) => {
                    assert_eq!(j.from, Position::new(0, 0));
                }
                _ => panic!("Expected Jump"),
            }
        }
    }

    mod player_input {
        use super::*;

        #[test]
        fn position_selected_holds_position() {
            let input = PlayerInput::PositionSelected(Position::new(3, 4));
            match input {
                PlayerInput::PositionSelected(pos) => {
                    assert_eq!(pos, Position::new(3, 4));
                }
                _ => panic!("Expected PositionSelected"),
            }
        }

        #[test]
        fn cancel_variant_exists() {
            let input = PlayerInput::Cancel;
            assert!(matches!(input, PlayerInput::Cancel));
        }
    }
}
