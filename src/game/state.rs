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
