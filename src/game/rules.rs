use crate::game::state::*;

#[derive(Debug, Clone)]
pub struct Jump {
    pub from: Position,
    pub to: Position,
    #[allow(dead_code)]
    pub direction: Direction,
    pub captured: Vec<Position>,
}

pub struct Rules;

impl Rules {
    // Opening phase: Black's valid removal positions (centers and corners with black pieces)
    pub fn valid_black_opening_removals(state: &GameState) -> Vec<Position> {
        let mut positions = Vec::new();

        // Center positions
        for pos in state.board.center_positions() {
            if state.board.get_piece_color(pos) == Some(PieceColor::Black) {
                positions.push(pos);
            }
        }

        // Corner positions
        for pos in state.board.corner_positions() {
            if state.board.get_piece_color(pos) == Some(PieceColor::Black) {
                positions.push(pos);
            }
        }

        positions
    }

    // Opening phase: White's valid removal positions (white pieces adjacent to empty)
    pub fn valid_white_opening_removals(state: &GameState) -> Vec<Position> {
        let mut positions = Vec::new();

        if let Some(empty_pos) = state.first_removal_pos {
            for neighbor in state.board.orthogonal_neighbors(empty_pos) {
                if state.board.get_piece_color(neighbor) == Some(PieceColor::White) {
                    positions.push(neighbor);
                }
            }
        }

        positions
    }

    // Check if a single jump is valid
    fn is_valid_single_jump(
        board: &Board,
        from: Position,
        direction: Direction,
        player: PieceColor,
    ) -> Option<(Position, Position)> {
        // Check if there's an opponent piece to jump over
        let over = direction.apply(from, board.size())?;
        if board.get_piece_color(over) != Some(player.opposite()) {
            return None;
        }

        // Check if the landing position is empty
        let to = direction.apply(over, board.size())?;
        if !board.is_empty(to) {
            return None;
        }

        Some((over, to))
    }

    // Get all possible jumps for a piece at a given position
    pub fn valid_jumps_from(state: &GameState, from: Position) -> Vec<Jump> {
        let board = &state.board;
        let player = state.current_player;

        // Must be player's piece
        if board.get_piece_color(from) != Some(player) {
            return Vec::new();
        }

        let mut jumps = Vec::new();

        for direction in Direction::all() {
            if let Some((captured_pos, to)) =
                Self::is_valid_single_jump(board, from, direction, player)
            {
                // Single jump
                jumps.push(Jump {
                    from,
                    to,
                    direction,
                    captured: vec![captured_pos],
                });

                // Multi-jumps in the same direction
                let mut current_to = to;
                let mut captured = vec![captured_pos];
                let mut test_board = board.clone();
                test_board.remove(from);
                test_board.remove(captured_pos);
                test_board.set(to, Cell::Occupied(player));

                loop {
                    // For multi-jump, we need to check from current_to
                    // First reset the test board state for checking
                    test_board.set(current_to, Cell::Empty);

                    if let Some((next_captured, next_to)) =
                        Self::is_valid_single_jump(&test_board, current_to, direction, player)
                    {
                        // Restore piece and update for next iteration
                        test_board.set(current_to, Cell::Occupied(player));
                        test_board.remove(next_captured);
                        test_board.set(next_to, Cell::Occupied(player));

                        captured.push(next_captured);
                        current_to = next_to;

                        jumps.push(Jump {
                            from,
                            to: current_to,
                            direction,
                            captured: captured.clone(),
                        });
                    } else {
                        break;
                    }
                }
            }
        }

        jumps
    }

    // Get all valid jumps for the current player
    pub fn all_valid_jumps(state: &GameState) -> Vec<Jump> {
        let mut jumps = Vec::new();
        let size = state.board.size();

        for row in 0..size {
            for col in 0..size {
                let pos = Position::new(row, col);
                jumps.extend(Self::valid_jumps_from(state, pos));
            }
        }

        jumps
    }

    // Check if the current player has any valid moves
    pub fn has_valid_move(state: &GameState) -> bool {
        match state.phase {
            GamePhase::OpeningBlackRemoval => !Self::valid_black_opening_removals(state).is_empty(),
            GamePhase::OpeningWhiteRemoval => !Self::valid_white_opening_removals(state).is_empty(),
            GamePhase::Play => !Self::all_valid_jumps(state).is_empty(),
            _ => false,
        }
    }

    // Get pieces that can move (have valid jumps)
    pub fn movable_pieces(state: &GameState) -> Vec<Position> {
        let mut pieces = Vec::new();
        let size = state.board.size();

        for row in 0..size {
            for col in 0..size {
                let pos = Position::new(row, col);
                if !Self::valid_jumps_from(state, pos).is_empty() {
                    pieces.push(pos);
                }
            }
        }

        pieces
    }

    // Apply a jump to the game state
    pub fn apply_jump(state: &mut GameState, jump: &Jump) {
        let player = state.current_player;

        // Move the piece
        state.board.remove(jump.from);
        state.board.set(jump.to, Cell::Occupied(player));

        // Remove captured pieces
        for &captured_pos in &jump.captured {
            state.board.remove(captured_pos);
        }

        // Record the move
        state.move_history.push(MoveRecord::Jump {
            color: player,
            from: jump.from,
            to: jump.to,
            captured: jump.captured.clone(),
        });

        // Switch player
        state.current_player = player.opposite();

        // Check if next player can move
        if !Self::has_valid_move(state) {
            state.phase = GamePhase::GameOver { winner: player };
        }
    }

    // Apply opening removal
    pub fn apply_opening_removal(state: &mut GameState, pos: Position) -> Result<(), &'static str> {
        match state.phase {
            GamePhase::OpeningBlackRemoval => {
                if !Self::valid_black_opening_removals(state).contains(&pos) {
                    return Err("Invalid removal position for Black");
                }
                state.board.remove(pos);
                state.first_removal_pos = Some(pos);
                state.move_history.push(MoveRecord::OpeningRemoval {
                    color: PieceColor::Black,
                    position: pos,
                });
                state.phase = GamePhase::OpeningWhiteRemoval;
                state.current_player = PieceColor::White;
            }
            GamePhase::OpeningWhiteRemoval => {
                if !Self::valid_white_opening_removals(state).contains(&pos) {
                    return Err("Invalid removal position for White");
                }
                state.board.remove(pos);
                state.move_history.push(MoveRecord::OpeningRemoval {
                    color: PieceColor::White,
                    position: pos,
                });
                state.phase = GamePhase::Play;
                state.current_player = PieceColor::Black;

                // Check if Black can move
                if !Self::has_valid_move(state) {
                    state.phase = GamePhase::GameOver {
                        winner: PieceColor::White,
                    };
                }
            }
            _ => return Err("Not in opening phase"),
        }
        Ok(())
    }
}
