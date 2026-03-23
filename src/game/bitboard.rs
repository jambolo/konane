//! 256-bit board-set representation: one `u16` per rank, bit `c` of rank `r` is the lua at (r, c).

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use crate::game::state::{Direction, Position};

/// Maximum board size supported.
pub const MAX_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Bitboard([u16; MAX_SIZE]);

/// Rows alternating between `first` and `!first`.
const fn alternating(first: u16) -> Bitboard {
    let mut rows = [0; MAX_SIZE];
    let mut r = 0;
    while r < MAX_SIZE {
        rows[r] = if r % 2 == 0 { first } else { !first };
        r += 1;
    }
    Bitboard(rows)
}

/// `BOARDS[n]` is every lua of an n×n board.
const BOARDS: [Bitboard; MAX_SIZE + 1] = {
    let mut boards = [Bitboard::EMPTY; MAX_SIZE + 1];
    let mut n = 0;
    while n <= MAX_SIZE {
        let row = ((1u32 << n) - 1) as u16;
        let mut r = 0;
        while r < n {
            boards[n].0[r] = row;
            r += 1;
        }
        n += 1;
    }
    boards
};

impl Bitboard {
    pub const EMPTY: Self = Self([0; MAX_SIZE]);

    /// Lua where (row + col) is even
    pub const EVEN: Self = alternating(0x5555);

    /// Lua where (row + col) is odd
    pub const ODD: Self = alternating(0xAAAA);

    /// Every lua of a `size`×`size` board.
    pub fn board(size: usize) -> Self {
        BOARDS[size]
    }

    /// The set containing only `pos`, which must be within `MAX_SIZE`.
    pub fn from_position(pos: Position) -> Self {
        let mut rows = [0; MAX_SIZE];
        rows[pos.row] = 1 << pos.col;
        Self(rows)
    }

    pub fn contains(self, pos: Position) -> bool {
        pos.row < MAX_SIZE && pos.col < MAX_SIZE && self.0[pos.row] & (1 << pos.col) != 0
    }

    pub fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    pub fn count(self) -> u32 {
        self.0.iter().map(|r| r.count_ones()).sum()
    }

    /// Moves every lua one step in `direction`. Lua shifted past row or column 15 are dropped.
    pub fn shift(self, direction: Direction) -> Self {
        let mut rows = [0; MAX_SIZE];
        match direction {
            Direction::Right => return Self(self.0.map(|r| r << 1)),
            Direction::Left => return Self(self.0.map(|r| r >> 1)),
            Direction::Up => rows[1..].copy_from_slice(&self.0[..MAX_SIZE - 1]),
            Direction::Down => rows[..MAX_SIZE - 1].copy_from_slice(&self.0[1..]),
        }
        Self(rows)
    }

    /// Positions in the set, in row-major order from a1.
    pub fn positions(self) -> impl Iterator<Item = Position> {
        self.0.into_iter().enumerate().flat_map(|(row, mut bits)| {
            std::iter::from_fn(move || {
                (bits != 0).then(|| {
                    let col = bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    Position::new(row, col)
                })
            })
        })
    }
}

macro_rules! bitwise {
    ($op:ident, $f:ident, $op_assign:ident, $f_assign:ident, $tok:tt) => {
        impl $op for Bitboard {
            type Output = Self;
            fn $f(self, rhs: Self) -> Self {
                Self(std::array::from_fn(|i| self.0[i] $tok rhs.0[i]))
            }
        }

        impl $op_assign for Bitboard {
            fn $f_assign(&mut self, rhs: Self) {
                *self = *self $tok rhs;
            }
        }
    };
}

bitwise!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
bitwise!(BitOr, bitor, BitOrAssign, bitor_assign, |);
bitwise!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

impl Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.map(|r| !r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(positions: &[(usize, usize)]) -> Bitboard {
        positions.iter().fold(Bitboard::EMPTY, |b, &(r, c)| b | Bitboard::from_position(Position::new(r, c)))
    }

    #[test]
    fn parity_masks_partition_the_board() {
        assert!((Bitboard::EVEN & Bitboard::ODD).is_empty());
        assert_eq!((Bitboard::EVEN | Bitboard::ODD).count(), 256);
        assert!(Bitboard::EVEN.contains(Position::new(0, 0)));
        assert!(Bitboard::EVEN.contains(Position::new(1, 1)));
        assert!(Bitboard::ODD.contains(Position::new(0, 1)));
        assert!(Bitboard::ODD.contains(Position::new(1, 0)));
    }

    #[test]
    fn board_covers_size_squared() {
        for size in (4..=16).step_by(2) {
            let b = Bitboard::board(size);
            assert_eq!(b.count() as usize, size * size);
            assert!(b.contains(Position::new(size - 1, size - 1)));
            assert!(!b.contains(Position::new(size, 0)));
            assert!(!b.contains(Position::new(0, size)));
        }
    }

    #[test]
    fn contains_rejects_out_of_range() {
        assert!(!Bitboard::board(16).contains(Position::new(16, 0)));
        assert!(!Bitboard::board(16).contains(Position::new(0, 16)));
    }

    #[test]
    fn shift_moves_one_step() {
        let b = set(&[(3, 3)]);
        assert_eq!(b.shift(Direction::Up), set(&[(4, 3)]));
        assert_eq!(b.shift(Direction::Down), set(&[(2, 3)]));
        assert_eq!(b.shift(Direction::Left), set(&[(3, 2)]));
        assert_eq!(b.shift(Direction::Right), set(&[(3, 4)]));
    }

    #[test]
    fn shift_drops_lua_off_the_edge() {
        assert!(set(&[(15, 0)]).shift(Direction::Up).is_empty());
        assert!(set(&[(0, 0)]).shift(Direction::Down).is_empty());
        assert!(set(&[(0, 0)]).shift(Direction::Left).is_empty());
        assert!(set(&[(0, 15)]).shift(Direction::Right).is_empty());
    }

    #[test]
    fn positions_are_row_major() {
        let b = set(&[(2, 5), (0, 3), (2, 1), (15, 15)]);
        let p: Vec<_> = b.positions().collect();
        assert_eq!(
            p,
            vec![Position::new(0, 3), Position::new(2, 1), Position::new(2, 5), Position::new(15, 15)]
        );
    }
}
