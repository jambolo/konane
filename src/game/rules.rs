use crate::game::bitboard::Bitboard;
use crate::game::state::*;

/// A straight-line jump of `length` captures from `from` in `direction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Jump {
    pub from: Position,
    pub direction: Direction,
    pub length: u8,
}

impl Jump {
    pub fn to(&self) -> Position {
        self.direction.step(self.from, 2 * self.length as usize)
    }

    pub fn captured(&self) -> impl Iterator<Item = Position> {
        let jump = *self;
        (0..jump.length as usize).map(move |i| jump.direction.step(jump.from, 2 * i + 1))
    }
}

pub struct Rules;

impl Rules {
    // Opening phase: Black's valid removal positions (centers and corners with black pieces)
    pub fn valid_black_opening_removals(state: &KonaneState) -> Vec<Position> {
        // * Center positions -- black's center positions are always (N/2-1, N/2-1) and (N/2, N/2)
        // * Corner positions -- black's corner positions are always (0, 0) and (N-1, N-1)
        let board_size = state.board().size();
        vec![
            Position {
                row: board_size / 2 - 1,
                col: board_size / 2 - 1,
            },
            Position {
                row: board_size / 2,
                col: board_size / 2,
            },
            Position { row: 0, col: 0 },
            Position {
                row: board_size - 1,
                col: board_size - 1,
            },
        ]
    }

    // Opening phase: White's valid removal positions (white pieces adjacent to empty)
    pub fn valid_white_opening_removals(state: &KonaneState) -> Vec<Position> {
        let Some(empty_pos) = state.get_opening_position() else {
            return Vec::new();
        };
        let empty = Bitboard::from_position(empty_pos);
        let neighbors = Direction::all()
            .into_iter()
            .fold(Bitboard::EMPTY, |acc, d| acc | empty.shift(d));
        (neighbors & state.board().stones(PieceColor::White)).positions().collect()
    }

    /// Calls `f(direction, length, landings)` for every direction and jump length available to `movers`, where `landings` is the
    /// set of lua reached. Each landing lua has exactly one origin: `length` double steps back.
    fn for_each_landing(board: &Board, player: PieceColor, movers: Bitboard, mut f: impl FnMut(Direction, u8, Bitboard)) {
        let opponents = board.stones(player.opposite());
        let vacancies = board.vacancies(player);
        for direction in Direction::all() {
            let mut reach = movers;
            for length in 1u8.. {
                reach = (reach.shift(direction) & opponents).shift(direction) & vacancies;
                if reach.is_empty() {
                    break;
                }
                f(direction, length, reach);
            }
        }
    }

    /// Lua `player` can reach with a single jump in `direction`
    fn single_jump_landings(board: &Board, player: PieceColor, direction: Direction) -> Bitboard {
        (board.stones(player).shift(direction) & board.stones(player.opposite())).shift(direction) & board.vacancies(player)
    }

    fn collect_jumps(board: &Board, player: PieceColor, movers: Bitboard) -> Vec<Jump> {
        let mut jumps = Vec::new();
        Self::for_each_landing(board, player, movers, |direction, length, landings| {
            let back = direction.opposite();
            jumps.extend(landings.positions().map(|to| Jump {
                from: back.step(to, 2 * length as usize),
                direction,
                length,
            }));
        });
        jumps
    }

    // Get all possible jumps for a piece at a given position
    pub fn valid_jumps_from(state: &KonaneState, from: Position) -> Vec<Jump> {
        let board = state.board();
        let player = state.current_player();

        // Must be player's piece
        if !board.stones(player).contains(from) {
            return Vec::new();
        }

        Self::collect_jumps(board, player, Bitboard::from_position(from))
    }

    // Get all valid jumps for the current player
    pub fn all_valid_jumps(state: &KonaneState) -> Vec<Jump> {
        let board = state.board();
        let player = state.current_player();
        Self::collect_jumps(board, player, board.stones(player))
    }

    /// Number of jumps available to `player`
    pub fn mobility(board: &Board, player: PieceColor) -> u32 {
        let mut count = 0;
        Self::for_each_landing(board, player, board.stones(player), |_, _, landings| {
            count += landings.count()
        });
        count
    }

    // Check if the current player has any valid moves
    pub fn has_valid_move(state: &KonaneState) -> bool {
        match state.current_phase() {
            GamePhase::OpeningBlackRemoval => !Self::valid_black_opening_removals(state).is_empty(),
            GamePhase::OpeningWhiteRemoval => !Self::valid_white_opening_removals(state).is_empty(),
            // Every multi-jump begins with a single jump
            GamePhase::Play => Direction::all()
                .into_iter()
                .any(|d| !Self::single_jump_landings(state.board(), state.current_player(), d).is_empty()),
            _ => false,
        }
    }

    // Get pieces that can move (have valid jumps)
    pub fn movable_pieces(state: &KonaneState) -> Vec<Position> {
        let board = state.board();
        let player = state.current_player();
        let movable = Direction::all().into_iter().fold(Bitboard::EMPTY, |acc, d| {
            let back = d.opposite();
            acc | Self::single_jump_landings(board, player, d).shift(back).shift(back)
        });
        movable.positions().collect()
    }

    // Apply a jump to the game state, returns the move record
    pub fn apply_jump(state: &mut KonaneState, jump: &Jump) -> MoveRecord {
        let player = state.current_player();
        let to = jump.to();

        // Move the piece
        state.move_stone(jump.from, to);

        // Remove captured pieces
        for captured_pos in jump.captured() {
            state.remove_stone(captured_pos);
        }

        // Switch player
        state.end_turn();

        // Check if next player can move
        if !Self::has_valid_move(state) {
            state.change_phase(GamePhase::GameOver { winner: player });
        }

        MoveRecord::Jump {
            color: player,
            from: jump.from,
            to,
            captured: jump.captured().collect(),
        }
    }
    // Apply opening removal, returns the move record
    pub fn apply_opening_removal(state: &mut KonaneState, pos: Position) -> Result<MoveRecord, &'static str> {
        match state.current_phase() {
            GamePhase::OpeningBlackRemoval => {
                if !Self::valid_black_opening_removals(state).contains(&pos) {
                    return Err("Invalid removal position for Black");
                }
                state.remove_opening_stone(pos);
                state.change_phase(GamePhase::OpeningWhiteRemoval);
                state.end_turn();
                Ok(MoveRecord::OpeningRemoval {
                    color: PieceColor::Black,
                    position: pos,
                })
            }
            GamePhase::OpeningWhiteRemoval => {
                if !Self::valid_white_opening_removals(state).contains(&pos) {
                    return Err("Invalid removal position for White");
                }
                state.remove_stone(pos);
                state.change_phase(GamePhase::Play);
                state.end_turn();

                // Check if Black can move
                if !Self::has_valid_move(state) {
                    state.change_phase(GamePhase::GameOver {
                        winner: PieceColor::White,
                    });
                }
                Ok(MoveRecord::OpeningRemoval {
                    color: PieceColor::White,
                    position: pos,
                })
            }
            _ => Err("Not in opening phase"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_play_phase() -> KonaneState {
        let mut state = KonaneState::new(8, PieceColor::Black);
        // Remove center black piece
        let _ = Rules::apply_opening_removal(&mut state, Position::new(3, 3));
        // Remove adjacent white piece
        let _ = Rules::apply_opening_removal(&mut state, Position::new(3, 4));
        state
    }

    mod valid_black_opening_removals {
        use super::*;

        #[test]
        fn returns_center_and_corner_black_pieces() {
            let state = KonaneState::new(8, PieceColor::Black);
            let valid = Rules::valid_black_opening_removals(&state);

            // Center black pieces: (3,3) and (4,4) have even sum
            assert!(valid.contains(&Position::new(3, 3)));
            assert!(valid.contains(&Position::new(4, 4)));

            // Corner black pieces: (0,0) and (7,7) have even sum
            assert!(valid.contains(&Position::new(0, 0)));
            assert!(valid.contains(&Position::new(7, 7)));

            // White corners should not be included
            assert!(!valid.contains(&Position::new(0, 7)));
            assert!(!valid.contains(&Position::new(7, 0)));
        }

        #[test]
        fn only_black_pieces_on_4x4() {
            let state = KonaneState::new(4, PieceColor::Black);
            let valid = Rules::valid_black_opening_removals(&state);

            for pos in &valid {
                assert_eq!(state.board().get_piece_color(*pos), Some(PieceColor::Black));
            }
        }
    }

    mod valid_white_opening_removals {
        use super::*;

        #[test]
        fn returns_white_pieces_adjacent_to_removed() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            // Black removes d4 (3,3)
            let _ = Rules::apply_opening_removal(&mut state, Position::new(3, 3));

            let valid = Rules::valid_white_opening_removals(&state);

            // Adjacent white pieces: d3 (2,3), d5 (4,3), c4 (3,2), e4 (3,4)
            // * d3 (2,3): sum=5, odd -> White
            // * d5 (4,3): sum=7, odd -> White
            // * c4 (3,2): sum=5, odd -> White
            // * e4 (3,4): sum=7, odd -> White
            assert!(valid.contains(&Position::new(2, 3)));
            assert!(valid.contains(&Position::new(4, 3)));
            assert!(valid.contains(&Position::new(3, 2)));
            assert!(valid.contains(&Position::new(3, 4)));
        }

        #[test]
        fn returns_empty_before_black_removal() {
            let state = KonaneState::new(8, PieceColor::Black);
            let valid = Rules::valid_white_opening_removals(&state);
            assert!(valid.is_empty());
        }
    }

    mod valid_jumps_from {
        use super::*;

        #[test]
        fn returns_empty_for_wrong_color() {
            let state = setup_play_phase();
            // Try to get jumps for a white piece when it's black's turn
            let white_pos = Position::new(0, 1);
            let jumps = Rules::valid_jumps_from(&state, white_pos);
            assert!(jumps.is_empty());
        }

        #[test]
        fn returns_empty_for_empty_cell() {
            let state = setup_play_phase();
            // The removed positions are empty
            let jumps = Rules::valid_jumps_from(&state, Position::new(3, 3));
            assert!(jumps.is_empty());
        }

        #[test]
        fn finds_single_jump() {
            let _state = setup_play_phase();
            // Black at (3,5) can jump over white at (3,4)? No, (3,4) is now empty. We need a setup where a jump is possible. Black
            // at e4 (3,4) is empty, d4 (3,3) is empty Let's check if black at (3,2) can jump (3,2) is black, (3,3) is empty - no
            // jump possible there We need to find a position where black can jump over white into empty

            // After removal of d4 and e4: Black at c4 (3,2) - can it jump? Right: (3,3) is empty, no opponent to jump Let's check
            // black at b4 (3,1) which is white... Black at f4 (3,5) can jump left over e4 (3,4)? e4 is empty, not a valid jump

            // Actually, black pieces that might jump after removing d4 (black) and e4 (white): Black at f4 (3,5): look left - e4
            // (3,4) is empty - no jump Black at b4 (3,1)? That's white (sum=4, even - no, it's black!) Wait, (3,1) has sum=4, which
            // is even, so it's black. Black at b4 (3,1) looking right: c4 (3,2) is black, not opponent

            // Let me think more carefully. After opening: d4 (3,3) is empty (was black) e4 (3,4) is empty (was white) Black at c4
            // (3,2) can jump right over d4? No, d4 is empty Black at f4 (3,5) can jump left over e4? No, e4 is empty

            // We need to set up a specific scenario
            let mut state = KonaneState::new(4, PieceColor::Black);
            let _ = Rules::apply_opening_removal(&mut state, Position::new(1, 1)); // b2
            let _ = Rules::apply_opening_removal(&mut state, Position::new(1, 2)); // c2

            // Now: b2 empty, c2 empty Black at a2 (1,0) can jump right over b2? b2 is empty - no Black at d2 (1,3) can jump left
            // over c2? c2 is empty - no

            // We need opponent between piece and empty Let's manually set up the board
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2)); // c1 empty
            // Black at a1 (0,0), White at b1 (0,1), Empty at c1 (0,2) Black can jump from a1 over b1 to c1

            let jumps = Rules::valid_jumps_from(&state, Position::new(0, 0));
            assert_eq!(jumps.len(), 1);
            assert_eq!(jumps[0].from, Position::new(0, 0));
            assert_eq!(jumps[0].to(), Position::new(0, 2));
            assert_eq!(jumps[0].captured().collect::<Vec<_>>(), vec![Position::new(0, 1)]);
        }

        #[test]
        fn finds_multi_jump_in_same_direction() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            state.change_phase(GamePhase::Play);

            // Set up: Black at a1, White at b1, Empty at c1, White at d1, Empty at e1
            // * a1 (0,0) = Black
            // * b1 (0,1) = White
            // * c1 (0,2) = empty
            // * d1 (0,3) = White
            // * e1 (0,4) = empty
            state.remove_stone(Position::new(0, 2)); // c1 empty
            state.remove_stone(Position::new(0, 4)); // e1 empty

            let jumps = Rules::valid_jumps_from(&state, Position::new(0, 0));

            // Should find: a1->c1 (single) and a1->e1 (double)
            assert_eq!(jumps.len(), 2);

            let single = jumps.iter().find(|j| j.to() == Position::new(0, 2));
            assert!(single.is_some());
            assert_eq!(single.unwrap().captured().count(), 1);

            let double = jumps.iter().find(|j| j.to() == Position::new(0, 4));
            assert!(double.is_some());
            assert_eq!(double.unwrap().captured().count(), 2);
        }

        #[test]
        fn jumps_only_in_same_direction_for_multi() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            state.change_phase(GamePhase::Play);

            // Black at c3 (2,2), can jump right and also up
            // * Right: d3 (2,3) white, e3 (2,4) empty
            // * Up: c4 (3,2) white, c5 (4,2) empty
            state.remove_stone(Position::new(2, 4)); // e3 empty
            state.remove_stone(Position::new(4, 2)); // c5 empty

            let jumps = Rules::valid_jumps_from(&state, Position::new(2, 2));

            // Should have 2 separate jumps, not a combined multi-directional jump
            assert_eq!(jumps.len(), 2);

            for jump in &jumps {
                // Each jump captures only 1 piece
                assert_eq!(jump.captured().count(), 1);
            }
        }
    }

    mod all_valid_jumps {
        use super::*;

        #[test]
        fn collects_jumps_from_all_pieces() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2)); // c1 empty
            state.remove_stone(Position::new(2, 0)); // a3 empty

            let all = Rules::all_valid_jumps(&state);

            // Black at a1 can jump to c1, Black at a3 (2,0)? a3 is now empty Black at c3 (2,2) can jump to a3
            assert!(all.len() >= 2);
        }
    }

    mod has_valid_move {
        use super::*;

        #[test]
        fn true_during_opening_black() {
            let state = KonaneState::new(8, PieceColor::Black);
            assert!(Rules::has_valid_move(&state));
        }

        #[test]
        fn true_during_opening_white() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            let _ = Rules::apply_opening_removal(&mut state, Position::new(3, 3));
            assert!(Rules::has_valid_move(&state));
        }

        #[test]
        fn false_when_no_jumps_available() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);

            // Remove all white pieces so black can't jump
            for row in 0..4 {
                for col in 0..4 {
                    let pos = Position::new(row, col);
                    if state.board().get_piece_color(pos) == Some(PieceColor::White) {
                        state.remove_stone(pos);
                    }
                }
            }

            assert!(!Rules::has_valid_move(&state));
        }

        #[test]
        fn false_during_game_over() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::GameOver {
                winner: PieceColor::Black,
            });
            assert!(!Rules::has_valid_move(&state));
        }
    }

    mod movable_pieces {
        use super::*;

        #[test]
        fn returns_pieces_with_valid_jumps() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2)); // c1 empty

            let movable = Rules::movable_pieces(&state);

            // Only a1 (0,0) can jump
            assert!(movable.contains(&Position::new(0, 0)));
        }

        #[test]
        fn returns_empty_when_no_moves() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);

            // Remove all white pieces
            for row in 0..4 {
                for col in 0..4 {
                    let pos = Position::new(row, col);
                    if state.board().get_piece_color(pos) == Some(PieceColor::White) {
                        state.remove_stone(pos);
                    }
                }
            }

            assert!(Rules::movable_pieces(&state).is_empty());
        }
    }

    mod apply_jump {
        use super::*;

        #[test]
        fn moves_piece_to_destination() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2)); // c1 empty

            let jump = Jump {
                from: Position::new(0, 0),
                direction: Direction::Right,
                length: 1,
            };

            Rules::apply_jump(&mut state, &jump);

            assert!(state.board().is_empty(Position::new(0, 0)));
            assert_eq!(state.board().get_piece_color(Position::new(0, 2)), Some(PieceColor::Black));
        }

        #[test]
        fn removes_captured_pieces() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2)); // c1 empty

            let jump = Jump {
                from: Position::new(0, 0),
                direction: Direction::Right,
                length: 1,
            };

            Rules::apply_jump(&mut state, &jump);

            assert!(state.board().is_empty(Position::new(0, 1)));
        }

        #[test]
        fn switches_player() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2));

            let jump = Jump {
                from: Position::new(0, 0),
                direction: Direction::Right,
                length: 1,
            };

            Rules::apply_jump(&mut state, &jump);

            assert_eq!(state.current_player(), PieceColor::White);
        }

        #[test]
        fn returns_move_record() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            state.remove_stone(Position::new(0, 2));

            let jump = Jump {
                from: Position::new(0, 0),
                direction: Direction::Right,
                length: 1,
            };

            let record = Rules::apply_jump(&mut state, &jump);

            match record {
                MoveRecord::Jump {
                    color,
                    from,
                    to,
                    captured,
                } => {
                    assert_eq!(color, PieceColor::Black);
                    assert_eq!(from, Position::new(0, 0));
                    assert_eq!(to, Position::new(0, 2));
                    assert_eq!(captured.len(), 1);
                }
                _ => panic!("Expected Jump record"),
            }
        }

        #[test]
        fn ends_game_when_opponent_has_no_moves() {
            let mut state = KonaneState::new(4, PieceColor::Black);
            state.change_phase(GamePhase::Play);

            // Remove all white pieces except one that will be captured
            for row in 0..4 {
                for col in 0..4 {
                    let pos = Position::new(row, col);
                    if pos != Position::new(0, 1) && state.board().get_piece_color(pos) == Some(PieceColor::White) {
                        state.remove_stone(pos);
                    }
                }
            }
            state.remove_stone(Position::new(0, 2)); // c1 empty for jump landing

            let jump = Jump {
                from: Position::new(0, 0),
                direction: Direction::Right,
                length: 1,
            };

            Rules::apply_jump(&mut state, &jump);

            // White has no pieces left, game over
            assert!(matches!(
                state.current_phase(),
                GamePhase::GameOver {
                    winner: PieceColor::Black
                }
            ));
        }
    }

    mod apply_opening_removal {
        use super::*;

        #[test]
        fn black_removal_transitions_to_white_phase() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            let result = Rules::apply_opening_removal(&mut state, Position::new(3, 3));

            assert!(result.is_ok());
            assert_eq!(state.current_phase(), GamePhase::OpeningWhiteRemoval);
            assert_eq!(state.current_player(), PieceColor::White);
        }

        #[test]
        fn black_removal_sets_first_removal_pos() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            let pos = Position::new(3, 3);
            let _ = Rules::apply_opening_removal(&mut state, pos);

            assert_eq!(state.get_opening_position(), Some(pos));
        }

        #[test]
        fn black_removal_returns_move_record() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            let result = Rules::apply_opening_removal(&mut state, Position::new(3, 3));

            assert!(result.is_ok());
            match result.unwrap() {
                MoveRecord::OpeningRemoval { color, position } => {
                    assert_eq!(color, PieceColor::Black);
                    assert_eq!(position, Position::new(3, 3));
                }
                _ => panic!("Expected OpeningRemoval"),
            }
        }

        #[test]
        fn white_removal_transitions_to_play() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            let _ = Rules::apply_opening_removal(&mut state, Position::new(3, 3));
            let result = Rules::apply_opening_removal(&mut state, Position::new(3, 4));

            assert!(result.is_ok());
            assert_eq!(state.current_phase(), GamePhase::Play);
            assert_eq!(state.current_player(), PieceColor::Black);
        }

        #[test]
        fn rejects_invalid_black_position() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            // Position (0,1) is white, not valid for black removal
            let result = Rules::apply_opening_removal(&mut state, Position::new(0, 1));
            assert!(result.is_err());
        }

        #[test]
        fn rejects_invalid_white_position() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            let _ = Rules::apply_opening_removal(&mut state, Position::new(3, 3));
            // Position (0,0) is not adjacent to the removed piece
            let result = Rules::apply_opening_removal(&mut state, Position::new(0, 0));
            assert!(result.is_err());
        }

        #[test]
        fn rejects_during_play_phase() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            state.change_phase(GamePhase::Play);
            let result = Rules::apply_opening_removal(&mut state, Position::new(3, 3));
            assert!(result.is_err());
        }
    }

    mod integration {
        use super::*;

        #[test]
        fn complete_game_sequence() {
            let mut state = KonaneState::new(4, PieceColor::Black);

            // Opening: Black removes b2 (1,1)
            assert!(Rules::apply_opening_removal(&mut state, Position::new(1, 1)).is_ok());

            // Opening: White removes c2 (1,2)
            assert!(Rules::apply_opening_removal(&mut state, Position::new(1, 2)).is_ok());

            // Now in Play phase
            assert_eq!(state.current_phase(), GamePhase::Play);
            assert_eq!(state.current_player(), PieceColor::Black);

            // Check valid jumps exist
            let jumps = Rules::all_valid_jumps(&state);
            assert!(!jumps.is_empty());
        }

        #[test]
        fn multi_jump_captures_multiple_pieces() {
            let mut state = KonaneState::new(8, PieceColor::Black);
            state.change_phase(GamePhase::Play);

            // Set up a multi-jump scenario
            state.remove_stone(Position::new(0, 2)); // c1 empty
            state.remove_stone(Position::new(0, 4)); // e1 empty

            let jumps = Rules::valid_jumps_from(&state, Position::new(0, 0));
            let multi_jump = jumps.iter().find(|j| j.captured().count() == 2);

            assert!(multi_jump.is_some());
            let jump = multi_jump.unwrap();

            Rules::apply_jump(&mut state, jump);

            // Both b1 and d1 should be empty now
            assert!(state.board().is_empty(Position::new(0, 1)));
            assert!(state.board().is_empty(Position::new(0, 3)));

            // Piece should be at e1
            assert_eq!(state.board().get_piece_color(Position::new(0, 4)), Some(PieceColor::Black));
        }
    }

    mod bitboard_equivalence {
        use super::*;
        use rand::SeedableRng;
        use rand::seq::IndexedRandom;
        use rand_chacha::ChaCha8Rng;
        use std::collections::{BTreeSet, HashSet};

        type Key = (usize, usize, usize, usize);

        fn key(from: Position, to: Position) -> Key {
            (from.row, from.col, to.row, to.col)
        }

        /// Lua-by-lua reference generator
        fn naive_jumps(state: &KonaneState) -> BTreeSet<Key> {
            let board = state.board();
            let size = board.size();
            let player = state.current_player();
            let mut jumps = BTreeSet::new();
            for row in 0..size {
                for col in 0..size {
                    let from = Position::new(row, col);
                    if board.get_piece_color(from) != Some(player) {
                        continue;
                    }
                    for direction in Direction::all() {
                        let mut at = from;
                        while let Some(over) = direction.apply(at, size)
                            && let Some(land) = direction.apply(over, size)
                            && board.get_piece_color(over) == Some(player.opposite())
                            && board.is_empty(land)
                        {
                            jumps.insert(key(from, land));
                            at = land;
                        }
                    }
                }
            }
            jumps
        }

        fn check(state: &KonaneState) {
            let jumps = Rules::all_valid_jumps(state);
            let keys: BTreeSet<Key> = jumps.iter().map(|j| key(j.from, j.to())).collect();
            assert_eq!(keys.len(), jumps.len(), "duplicate jumps");
            assert_eq!(keys, naive_jumps(state));
            assert_eq!(Rules::mobility(state.board(), state.current_player()) as usize, jumps.len());
            assert_eq!(Rules::has_valid_move(state), !jumps.is_empty());

            let froms: HashSet<Position> = jumps.iter().map(|j| j.from).collect();
            assert_eq!(Rules::movable_pieces(state).into_iter().collect::<HashSet<_>>(), froms);

            for j in &jumps {
                for c in j.captured() {
                    assert_eq!(state.board().get_piece_color(c), Some(state.current_player().opposite()));
                }
                let from_here: Vec<Jump> = Rules::valid_jumps_from(state, j.from);
                assert!(from_here.contains(j));
            }
        }

        #[test]
        fn matches_naive_generator_in_random_games() {
            let mut rng = ChaCha8Rng::seed_from_u64(1);
            for size in (4..=16).step_by(2) {
                for _ in 0..20 {
                    let mut state = KonaneState::new(size, PieceColor::Black);
                    let black = *Rules::valid_black_opening_removals(&state).choose(&mut rng).unwrap();
                    Rules::apply_opening_removal(&mut state, black).unwrap();
                    let white = *Rules::valid_white_opening_removals(&state).choose(&mut rng).unwrap();
                    Rules::apply_opening_removal(&mut state, white).unwrap();

                    while state.current_phase() == GamePhase::Play {
                        check(&state);
                        let jumps = Rules::all_valid_jumps(&state);
                        let jump = *jumps.choose(&mut rng).unwrap();
                        let before = state.board().occupied().count();
                        Rules::apply_jump(&mut state, &jump);
                        assert_eq!(state.board().occupied().count(), before - jump.length as u32);
                    }
                    check(&state);
                }
            }
        }
    }
}
