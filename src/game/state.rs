use ndarray::Array2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PieceColor {
    Black,
    White,
}

impl PieceColor {
    pub fn opposite(&self) -> PieceColor {
        match self {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        }
    }
}

impl std::fmt::Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceColor::Black => write!(f, "Black"),
            PieceColor::White => write!(f, "White"),
        }
    }
}

/// Position on the board using algebraic notation conventions:
/// - row 0 is the bottom row (rank 1)
/// - col 0 is the leftmost column (file 'a')
/// - rows increase upward
/// - columns increase to the right
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Convert to algebraic notation (e.g., "a1", "e4")
    pub fn to_algebraic(self) -> String {
        let file = (b'a' + self.col as u8) as char;
        let rank = self.row + 1;
        format!("{}{}", file, rank)
    }

    /// Parse from algebraic notation (e.g., "a1", "e4")
    pub fn _from_algebraic(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();
        let mut chars = s.chars();
        let file = chars.next()?;
        let rank: usize = chars.collect::<String>().parse().ok()?;

        if !file.is_ascii_lowercase() || rank == 0 {
            return None;
        }

        let col = (file as u8 - b'a') as usize;
        let row = rank - 1;
        Some(Position::new(row, col))
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,    // Increasing row (toward higher ranks)
    Down,  // Decreasing row (toward lower ranks)
    Left,  // Decreasing col (toward 'a' file)
    Right, // Increasing col (toward higher files)
}

impl Direction {
    pub fn all() -> [Direction; 4] {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
    }

    pub fn apply(&self, pos: Position, board_size: usize) -> Option<Position> {
        match self {
            // Up increases row (toward higher ranks)
            Direction::Up if pos.row < board_size - 1 => Some(Position::new(pos.row + 1, pos.col)),
            // Down decreases row (toward lower ranks)
            Direction::Down if pos.row > 0 => Some(Position::new(pos.row - 1, pos.col)),
            // Left decreases col
            Direction::Left if pos.col > 0 => Some(Position::new(pos.row, pos.col - 1)),
            // Right increases col
            Direction::Right if pos.col < board_size - 1 => {
                Some(Position::new(pos.row, pos.col + 1))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Occupied(PieceColor),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Setup,
    OpeningBlackRemoval,
    OpeningWhiteRemoval,
    Play,
    GameOver { winner: PieceColor },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoveRecord {
    OpeningRemoval {
        color: PieceColor,
        position: Position,
    },
    Jump {
        color: PieceColor,
        from: Position,
        to: Position,
        captured: Vec<Position>,
    },
}

impl MoveRecord {
    /// Format move in algebraic notation
    pub fn to_algebraic(&self) -> String {
        match self {
            MoveRecord::OpeningRemoval { position, .. } => position.to_algebraic(),
            MoveRecord::Jump { from, to, .. } => {
                format!("{}-{}", from.to_algebraic(), to.to_algebraic())
            }
        }
    }
}

impl std::fmt::Display for MoveRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveRecord::OpeningRemoval { color, position } => {
                write!(f, "{} removes piece at {}", color, position)
            }
            MoveRecord::Jump {
                color, captured, ..
            } => {
                write!(
                    f,
                    "{} jumps {} capturing {} piece(s)",
                    color,
                    self.to_algebraic(),
                    captured.len()
                )
            }
        }
    }
}

/// Board representation using ndarray.
/// Coordinate system: (row, col) where (0, 0) is the bottom-left corner.
/// Row 0 is the bottom row, rows increase upward.
/// Col 0 is the leftmost column, cols increase to the right.
#[derive(Debug, Clone)]
pub struct Board {
    size: usize,
    cells: Array2<Cell>,
}

impl Board {
    pub fn new(size: usize) -> Self {
        assert!(
            (4..=16).contains(&size) && size.is_multiple_of(2),
            "Board size must be even, between 4 and 16"
        );

        // Initialize with checkerboard pattern
        // Per rules: "first lua contains a Black piece" - a1 (0,0) is Black
        let cells = Array2::from_shape_fn((size, size), |(row, col)| {
            // (0,0) = a1 = Black, checkerboard pattern
            let color = if (row + col) % 2 == 0 {
                PieceColor::Black
            } else {
                PieceColor::White
            };
            Cell::Occupied(color)
        });

        Self { size, cells }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get(&self, pos: Position) -> Option<Cell> {
        if pos.row < self.size && pos.col < self.size {
            Some(self.cells[[pos.row, pos.col]])
        } else {
            None
        }
    }

    pub fn set(&mut self, pos: Position, cell: Cell) {
        if pos.row < self.size && pos.col < self.size {
            self.cells[[pos.row, pos.col]] = cell;
        }
    }

    pub fn remove(&mut self, pos: Position) {
        self.set(pos, Cell::Empty);
    }

    pub fn is_empty(&self, pos: Position) -> bool {
        matches!(self.get(pos), Some(Cell::Empty))
    }

    pub fn get_piece_color(&self, pos: Position) -> Option<PieceColor> {
        match self.get(pos) {
            Some(Cell::Occupied(color)) => Some(color),
            _ => None,
        }
    }

    /// Returns the four center positions for the board.
    /// For an NxN board, the center is at positions (N/2-1, N/2-1), (N/2-1, N/2), (N/2, N/2-1), (N/2, N/2).
    pub fn center_positions(&self) -> Vec<Position> {
        let mid = self.size / 2;
        vec![
            Position::new(mid - 1, mid - 1),
            Position::new(mid - 1, mid),
            Position::new(mid, mid - 1),
            Position::new(mid, mid),
        ]
    }

    /// Returns the four corner positions.
    /// Note: On an even-sized board with checkerboard pattern starting with Black at (0,0):
    /// - (0, 0) and (size-1, size-1) are Black (even sum)
    /// - (0, size-1) and (size-1, 0) are White (odd sum, since size is even)
    pub fn corner_positions(&self) -> Vec<Position> {
        vec![
            Position::new(0, 0),                         // a1 - Black
            Position::new(0, self.size - 1),             // (a, size) - White
            Position::new(self.size - 1, 0),             // (size, a) - White
            Position::new(self.size - 1, self.size - 1), // (size, size) - Black
        ]
    }

    pub fn orthogonal_neighbors(&self, pos: Position) -> Vec<Position> {
        Direction::all()
            .iter()
            .filter_map(|d| d.apply(pos, self.size))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub board: Board,
    pub phase: GamePhase,
    pub current_player: PieceColor,
    pub move_history: Vec<MoveRecord>,
    pub first_removal_pos: Option<Position>,
}

impl GameState {
    pub fn new(board_size: usize, _first_player: PieceColor) -> Self {
        // Note: first_player is recorded for future use (e.g., tracking which human is which color)
        // The game always starts with Black making the first opening removal per Kōnane rules
        Self {
            board: Board::new(board_size),
            phase: GamePhase::OpeningBlackRemoval,
            current_player: PieceColor::Black,
            move_history: Vec::new(),
            first_removal_pos: None,
        }
    }

    pub fn _board_size(&self) -> usize {
        self.board.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod piece_color {
        use super::*;

        #[test]
        fn opposite_of_black_is_white() {
            assert_eq!(PieceColor::Black.opposite(), PieceColor::White);
        }

        #[test]
        fn opposite_of_white_is_black() {
            assert_eq!(PieceColor::White.opposite(), PieceColor::Black);
        }

        #[test]
        fn display_formats_correctly() {
            assert_eq!(format!("{}", PieceColor::Black), "Black");
            assert_eq!(format!("{}", PieceColor::White), "White");
        }
    }

    mod position {
        use super::*;

        #[test]
        fn new_creates_position() {
            let pos = Position::new(3, 5);
            assert_eq!(pos.row, 3);
            assert_eq!(pos.col, 5);
        }

        #[test]
        fn to_algebraic_converts_origin() {
            assert_eq!(Position::new(0, 0).to_algebraic(), "a1");
        }

        #[test]
        fn to_algebraic_converts_various_positions() {
            assert_eq!(Position::new(0, 4).to_algebraic(), "e1");
            assert_eq!(Position::new(3, 3).to_algebraic(), "d4");
            assert_eq!(Position::new(7, 7).to_algebraic(), "h8");
        }

        #[test]
        fn from_algebraic_parses_valid_notation() {
            assert_eq!(Position::_from_algebraic("a1"), Some(Position::new(0, 0)));
            assert_eq!(Position::_from_algebraic("e4"), Some(Position::new(3, 4)));
            assert_eq!(Position::_from_algebraic("h8"), Some(Position::new(7, 7)));
        }

        #[test]
        fn from_algebraic_handles_uppercase() {
            assert_eq!(Position::_from_algebraic("A1"), Some(Position::new(0, 0)));
            assert_eq!(Position::_from_algebraic("E4"), Some(Position::new(3, 4)));
        }

        #[test]
        fn from_algebraic_rejects_invalid_input() {
            assert_eq!(Position::_from_algebraic(""), None);
            assert_eq!(Position::_from_algebraic("a0"), None);
            assert_eq!(Position::_from_algebraic("1a"), None);
            assert_eq!(Position::_from_algebraic("aa"), None);
        }

        #[test]
        fn from_algebraic_handles_double_digit_ranks() {
            assert_eq!(
                Position::_from_algebraic("a10"),
                Some(Position::new(9, 0))
            );
            assert_eq!(
                Position::_from_algebraic("p16"),
                Some(Position::new(15, 15))
            );
        }

        #[test]
        fn display_uses_algebraic() {
            let pos = Position::new(3, 4);
            assert_eq!(format!("{}", pos), "e4");
        }

        #[test]
        fn roundtrip_conversion() {
            for row in 0..8 {
                for col in 0..8 {
                    let pos = Position::new(row, col);
                    let algebraic = pos.to_algebraic();
                    let parsed = Position::_from_algebraic(&algebraic);
                    assert_eq!(parsed, Some(pos));
                }
            }
        }
    }

    mod direction {
        use super::*;

        #[test]
        fn all_returns_four_directions() {
            let dirs = Direction::all();
            assert_eq!(dirs.len(), 4);
            assert!(dirs.contains(&Direction::Up));
            assert!(dirs.contains(&Direction::Down));
            assert!(dirs.contains(&Direction::Left));
            assert!(dirs.contains(&Direction::Right));
        }

        #[test]
        fn apply_up_increases_row() {
            let pos = Position::new(3, 3);
            let result = Direction::Up.apply(pos, 8);
            assert_eq!(result, Some(Position::new(4, 3)));
        }

        #[test]
        fn apply_down_decreases_row() {
            let pos = Position::new(3, 3);
            let result = Direction::Down.apply(pos, 8);
            assert_eq!(result, Some(Position::new(2, 3)));
        }

        #[test]
        fn apply_left_decreases_col() {
            let pos = Position::new(3, 3);
            let result = Direction::Left.apply(pos, 8);
            assert_eq!(result, Some(Position::new(3, 2)));
        }

        #[test]
        fn apply_right_increases_col() {
            let pos = Position::new(3, 3);
            let result = Direction::Right.apply(pos, 8);
            assert_eq!(result, Some(Position::new(3, 4)));
        }

        #[test]
        fn apply_returns_none_at_boundaries() {
            assert_eq!(Direction::Up.apply(Position::new(7, 3), 8), None);
            assert_eq!(Direction::Down.apply(Position::new(0, 3), 8), None);
            assert_eq!(Direction::Left.apply(Position::new(3, 0), 8), None);
            assert_eq!(Direction::Right.apply(Position::new(3, 7), 8), None);
        }
    }

    mod board {
        use super::*;

        #[test]
        fn new_creates_board_with_correct_size() {
            let board = Board::new(8);
            assert_eq!(board.size(), 8);
        }

        #[test]
        #[should_panic(expected = "Board size must be even")]
        fn new_rejects_odd_size() {
            Board::new(7);
        }

        #[test]
        #[should_panic(expected = "Board size must be even")]
        fn new_rejects_size_too_small() {
            Board::new(2);
        }

        #[test]
        #[should_panic(expected = "Board size must be even")]
        fn new_rejects_size_too_large() {
            Board::new(18);
        }

        #[test]
        fn checkerboard_pattern_a1_is_black() {
            let board = Board::new(8);
            assert_eq!(
                board.get_piece_color(Position::new(0, 0)),
                Some(PieceColor::Black)
            );
        }

        #[test]
        fn checkerboard_pattern_alternates() {
            let board = Board::new(8);
            // Black at (0,0), White at (0,1), Black at (0,2)
            assert_eq!(
                board.get_piece_color(Position::new(0, 0)),
                Some(PieceColor::Black)
            );
            assert_eq!(
                board.get_piece_color(Position::new(0, 1)),
                Some(PieceColor::White)
            );
            assert_eq!(
                board.get_piece_color(Position::new(0, 2)),
                Some(PieceColor::Black)
            );
            assert_eq!(
                board.get_piece_color(Position::new(1, 0)),
                Some(PieceColor::White)
            );
            assert_eq!(
                board.get_piece_color(Position::new(1, 1)),
                Some(PieceColor::Black)
            );
        }

        #[test]
        fn corner_colors_on_8x8() {
            let board = Board::new(8);
            // a1 (0,0) = Black
            assert_eq!(
                board.get_piece_color(Position::new(0, 0)),
                Some(PieceColor::Black)
            );
            // h1 (0,7) = White
            assert_eq!(
                board.get_piece_color(Position::new(0, 7)),
                Some(PieceColor::White)
            );
            // a8 (7,0) = White
            assert_eq!(
                board.get_piece_color(Position::new(7, 0)),
                Some(PieceColor::White)
            );
            // h8 (7,7) = Black
            assert_eq!(
                board.get_piece_color(Position::new(7, 7)),
                Some(PieceColor::Black)
            );
        }

        #[test]
        fn get_returns_none_for_out_of_bounds() {
            let board = Board::new(8);
            assert_eq!(board.get(Position::new(8, 0)), None);
            assert_eq!(board.get(Position::new(0, 8)), None);
        }

        #[test]
        fn set_and_get() {
            let mut board = Board::new(4);
            let pos = Position::new(1, 1);
            board.set(pos, Cell::Empty);
            assert_eq!(board.get(pos), Some(Cell::Empty));
        }

        #[test]
        fn remove_makes_cell_empty() {
            let mut board = Board::new(4);
            let pos = Position::new(1, 1);
            assert!(!board.is_empty(pos));
            board.remove(pos);
            assert!(board.is_empty(pos));
        }

        #[test]
        fn center_positions_for_4x4() {
            let board = Board::new(4);
            let centers = board.center_positions();
            assert_eq!(centers.len(), 4);
            assert!(centers.contains(&Position::new(1, 1)));
            assert!(centers.contains(&Position::new(1, 2)));
            assert!(centers.contains(&Position::new(2, 1)));
            assert!(centers.contains(&Position::new(2, 2)));
        }

        #[test]
        fn center_positions_for_8x8() {
            let board = Board::new(8);
            let centers = board.center_positions();
            assert_eq!(centers.len(), 4);
            assert!(centers.contains(&Position::new(3, 3)));
            assert!(centers.contains(&Position::new(3, 4)));
            assert!(centers.contains(&Position::new(4, 3)));
            assert!(centers.contains(&Position::new(4, 4)));
        }

        #[test]
        fn corner_positions() {
            let board = Board::new(8);
            let corners = board.corner_positions();
            assert_eq!(corners.len(), 4);
            assert!(corners.contains(&Position::new(0, 0)));
            assert!(corners.contains(&Position::new(0, 7)));
            assert!(corners.contains(&Position::new(7, 0)));
            assert!(corners.contains(&Position::new(7, 7)));
        }

        #[test]
        fn orthogonal_neighbors_in_center() {
            let board = Board::new(8);
            let neighbors = board.orthogonal_neighbors(Position::new(3, 3));
            assert_eq!(neighbors.len(), 4);
            assert!(neighbors.contains(&Position::new(4, 3))); // Up
            assert!(neighbors.contains(&Position::new(2, 3))); // Down
            assert!(neighbors.contains(&Position::new(3, 2))); // Left
            assert!(neighbors.contains(&Position::new(3, 4))); // Right
        }

        #[test]
        fn orthogonal_neighbors_at_corner() {
            let board = Board::new(8);
            let neighbors = board.orthogonal_neighbors(Position::new(0, 0));
            assert_eq!(neighbors.len(), 2);
            assert!(neighbors.contains(&Position::new(1, 0))); // Up
            assert!(neighbors.contains(&Position::new(0, 1))); // Right
        }
    }

    mod move_record {
        use super::*;

        #[test]
        fn opening_removal_to_algebraic() {
            let record = MoveRecord::OpeningRemoval {
                color: PieceColor::Black,
                position: Position::new(3, 4),
            };
            assert_eq!(record.to_algebraic(), "e4");
        }

        #[test]
        fn jump_to_algebraic() {
            let record = MoveRecord::Jump {
                color: PieceColor::White,
                from: Position::new(0, 0),
                to: Position::new(0, 2),
                captured: vec![Position::new(0, 1)],
            };
            assert_eq!(record.to_algebraic(), "a1-c1");
        }

        #[test]
        fn display_opening_removal() {
            let record = MoveRecord::OpeningRemoval {
                color: PieceColor::Black,
                position: Position::new(3, 4),
            };
            let display = format!("{}", record);
            assert!(display.contains("Black"));
            assert!(display.contains("e4"));
        }

        #[test]
        fn display_jump() {
            let record = MoveRecord::Jump {
                color: PieceColor::White,
                from: Position::new(0, 0),
                to: Position::new(0, 4),
                captured: vec![Position::new(0, 1), Position::new(0, 3)],
            };
            let display = format!("{}", record);
            assert!(display.contains("White"));
            assert!(display.contains("2 piece"));
        }
    }

    mod game_state {
        use super::*;

        #[test]
        fn new_starts_with_black_opening_removal() {
            let state = GameState::new(8, PieceColor::Black);
            assert_eq!(state.phase, GamePhase::OpeningBlackRemoval);
            assert_eq!(state.current_player, PieceColor::Black);
        }

        #[test]
        fn new_has_empty_history() {
            let state = GameState::new(8, PieceColor::Black);
            assert!(state.move_history.is_empty());
        }

        #[test]
        fn new_has_no_first_removal() {
            let state = GameState::new(8, PieceColor::Black);
            assert!(state.first_removal_pos.is_none());
        }

        #[test]
        fn board_size_accessor() {
            let state = GameState::new(6, PieceColor::Black);
            assert_eq!(state._board_size(), 6);
        }
    }
}
