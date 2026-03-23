# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Kōnane Design

The full rules of the game (authoritative reference for any rules question) are in [rules.md](rules.md).

### Architecture

**Why the game/rules/UI separation:**

- Game state can be serialized/deserialized independently
- Rules can be unit tested without UI
- Different UIs (CLI, web) could use the same game/rules modules
- AI players can query game state and rules without touching UI

Zobrist hashing (`game/zhash.rs`) is the transposition-table key; its RNG is `ChaCha8` so hashes are reproducible across runs.

**Board representation:** `Board` stores only an occupancy `Bitboard` (`game/bitboard.rs`, one `u16` per rank). Jumps move two lua, so a stone never changes lua parity and its color is implied by `(row + col) % 2`. Move generation in `rules.rs` shifts whole sets: landings = `shift(shift(own, d) & opponents, d) & vacancies`, repeated for multi-jumps on the unmodified board. `Jump` is `{from, direction, length}`; `to()` and `captured()` are derived.

### Coordinate System

Uses algebraic notation conventions matching the rules specification:

- Position(row, col) where (0,0) is the **bottom-left corner** (a1)
- Row increases **upward** (rank 1, 2, 3...)
- Column increases **rightward** (file a, b, c...)

### Board Size Constraints

- Supported sizes: **4×4 to 16×16, even sizes only**. The setup view enforces this; rules and AI code assume even-sided square boards.

### Initial Stone Placement

- Checkerboard pattern: `(row + col) % 2 == 0` is Black
- Position a1 (0,0) is always Black per rules ("first lua contains a Black piece")

### Move Logging

- Result codes: `1-0` = Black wins, `0-1` = White wins — Black is listed first, the opposite of the chess convention.

### Known Limitations / Future Work

1. **No network play** - Would need to serialize moves over socket
