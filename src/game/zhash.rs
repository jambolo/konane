//! Zobrist hashing for game state.

use crate::game::state::{Board, Cell, GamePhase, PieceColor, Position};
use std::sync::LazyLock;

/// Type alias for Zobrist hash values.
pub type Z = u64;

/// The initial value of a game.
const INITIAL_VALUE: Z = 0;

/// Maximum board size supported.
const MAX_SIZE: usize = 16;

/// Maximum number of positions (16x16).
const MAX_POSITIONS: usize = MAX_SIZE * MAX_SIZE;

/// Number of game phases.
const NUM_PHASES: usize = 6;

/// Simple XorShift64 PRNG.
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

/// Random number tables for Zobrist hashing.
struct ZobristTables {
    pieces: [u64; MAX_POSITIONS],
    turn: u64,
    phases: [u64; NUM_PHASES],
}

impl ZobristTables {
    fn new() -> Self {
        let mut rng = XorShift64::new(0x12345678_9ABCDEF0);

        let mut pieces = [0u64; MAX_POSITIONS];
        for pos in &mut pieces {
            *pos = rng.next();
        }

        let turn = rng.next();

        let mut phases = [0u64; NUM_PHASES];
        for phase in &mut phases {
            *phase = rng.next();
        }

        Self { pieces, turn, phases }
    }
}

static TABLES: LazyLock<ZobristTables> = LazyLock::new(ZobristTables::new);

fn pos_to_index(pos: Position) -> usize {
    pos.row * MAX_SIZE + pos.col
}

fn phase_to_index(phase: &GamePhase) -> usize {
    match phase {
        GamePhase::Setup => 0,
        GamePhase::OpeningBlackRemoval => 1,
        GamePhase::OpeningWhiteRemoval => 2,
        GamePhase::Play => 3,
        GamePhase::GameOver { winner: PieceColor::Black } => 4,
        GamePhase::GameOver { winner: PieceColor::White } => 5,
    }
}

/// Zobrist hash for incremental game state hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZHash {
    value: Z,
}

impl ZHash {
    /// Creates a new hash from a complete game state.
    pub fn from_state(board: &Board, phase: &GamePhase, turn: PieceColor) -> Self {
        let mut value: Z = 0;

        let size = board.size();
        for row in 0..size {
            for col in 0..size {
                let pos = Position::new(row, col);
                if let Some(Cell::Occupied(_)) = board.get(pos) {
                    value ^= TABLES.pieces[pos_to_index(pos)];
                }
            }
        }

        if turn == PieceColor::White {
            value ^= TABLES.turn;
        }

        value ^= TABLES.phases[phase_to_index(phase)];
        Self { value }
    }

    /// Creates a hash with initial value.
    pub fn new() -> Self {
        Self { value: INITIAL_VALUE }
    }

    /// Returns the current hash value.
    pub fn value(&self) -> Z {
        self.value
    }

    /// Updates the hash after removing a piece.
    pub fn remove_stone(&mut self, pos: Position) -> &mut Self {
        self.value ^= TABLES.pieces[pos_to_index(pos)];
        self
    }

    /// Updates the hash after moving a piece.
    pub fn move_stone(&mut self, from: Position, to: Position) -> &mut Self {
        self.value ^= TABLES.pieces[pos_to_index(from)];
        self.value ^= TABLES.pieces[pos_to_index(to)];
        self
    }

    /// Updates the hash after changing turn.
    pub fn end_turn(&mut self) -> &mut Self {
        self.value ^= TABLES.turn;
        self
    }

    /// Updates the hash after changing game phase.
    pub fn change_phase(&mut self, old: &GamePhase, new: &GamePhase) -> &mut Self {
        self.value ^= TABLES.phases[phase_to_index(old)];
        self.value ^= TABLES.phases[phase_to_index(new)];
        self
    }
}

impl Default for ZHash {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_initial_hash() {
        let hash = ZHash::new();
        assert_eq!(hash.value(), INITIAL_VALUE);
    }

    #[test]
    fn from_state_creates_defined_hash() {
        let board = Board::new(8);
        let hash = ZHash::from_state(&board, &GamePhase::Play, PieceColor::Black);
        assert_ne!(hash.value(), INITIAL_VALUE);
    }

    #[test]
    fn remove_is_reversible() {
        let board = Board::new(8);
        let original = ZHash::from_state(&board, &GamePhase::Play, PieceColor::Black);
        let mut hash = original;
        let pos = Position::new(0, 0);
        hash.remove_stone(pos);
        assert_ne!(hash.value(), original.value());
        hash.remove_stone(pos);
        assert_eq!(hash.value(), original.value());
    }

    #[test]
    fn move_stone_is_reversible() {
        let board = Board::new(8);
        let original = ZHash::from_state(&board, &GamePhase::Play, PieceColor::Black);
        let mut hash = original;
        let from = Position::new(0, 0);
        let to = Position::new(0, 2);
        hash.move_stone(from, to);
        assert_ne!(hash.value(), original.value());
        hash.move_stone(to, from);
        assert_eq!(hash.value(), original.value());
    }

    #[test]
    fn turn_toggle_is_reversible() {
        let board = Board::new(8);
        let original = ZHash::from_state(&board, &GamePhase::Play, PieceColor::Black);
        let mut hash = original;
        hash.end_turn();
        assert_ne!(hash.value(), original.value());
        hash.end_turn();
        assert_eq!(hash.value(), original.value());
    }

    #[test]
    fn change_phase_is_reversible() {
        let board = Board::new(8);
        let original = ZHash::from_state(&board, &GamePhase::Play, PieceColor::Black);
        let mut hash = original;
        hash.change_phase(&GamePhase::Play, &GamePhase::GameOver { winner: PieceColor::Black });
        assert_ne!(hash.value(), original.value());
        hash.change_phase(&GamePhase::GameOver { winner: PieceColor::Black }, &GamePhase::Play);
        assert_eq!(hash.value(), original.value());
    }

    #[test]
    fn different_turns_produce_different_hashes() {
        let board = Board::new(8);
        let hash_black = ZHash::from_state(&board, &GamePhase::Play, PieceColor::Black);
        let hash_white = ZHash::from_state(&board, &GamePhase::Play, PieceColor::White);
        assert_ne!(hash_black.value(), hash_white.value());
    }

    #[test]
    fn different_phases_produce_different_hashes() {
        let board = Board::new(8);
        let hash1 = ZHash::from_state(&board, &GamePhase::Play, PieceColor::Black);
        let hash2 = ZHash::from_state(&board, &GamePhase::OpeningBlackRemoval, PieceColor::Black);
        assert_ne!(hash1.value(), hash2.value());
    }
}
