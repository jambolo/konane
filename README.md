# Kōnane

![Rust Workflow](https://github.com/jambolo/game-player/actions/workflows/rust.yml/badge.svg)

Kōnane is a traditional Hawaiian strategy board game.

## Features

- Configurable board sizes (4x4 to 16x16)
- Person vs person, person vs computer, or computer vs computer
- Undo/Redo support
- Game import/export (JSON and text formats)
- Move history with algebraic notation

## Game Description

- Played on a rectangular grid (typically square, even-sized).
- Two players: Black (lava stone) and White (coral).
- Board is filled in a checkerboard pattern; all cells occupied at start.

## Rules

### Setup & Opening

1. Board is initialized with alternating Black and White pieces; a1 (bottom-left) is always Black.
2. Black removes one Black stone from the center or a corner.
3. White removes one White stone orthogonally adjacent to the empty space.

### Movement & Removal

- Players alternate turns.
- All moves are jumps that remove a stone; there are no other moves.
- Jumps are orthogonal only (up, down, left, right).
- Jump over an opponent’s stone into an empty space; remove the jumped stone.
- Multi-jumps allowed in a straight line (no direction change); optional to continue.

### Win Condition

- If a player cannot make a legal jump, the game ends.
- The last player to make a move wins.

For full rules and examples, see [rules.md](rules.md).

## Playing

### Setup

From the setup screen, configure:

- **Board size**: 4x4 to 16x16 (even sizes only)
- **Black**: Person or Computer
- **White**: Person or Computer

## JSON Game Format

Games can be exported and imported using JSON. The format:

```json
{
  "board_size": 8,
  "winner": "Black",
  "total_moves": 42,
  "moves": [
    { "OpeningRemoval": { "color": "Black", "position": { "row": 3, "col": 3 } } },
    { "OpeningRemoval": { "color": "White", "position": { "row": 3, "col": 4 } } },
    { "Jump": { "color": "Black", "from": { "row": 3, "col": 5 }, "to": { "row": 3, "col": 3 }, "removed": [{ "row": 3, "col": 4 }] } }
  ]
}
```

- `board_size`: Board dimension (4-16, must be even)
- `winner`: (optional) "Black" or "White" - only present for completed games
- `total_moves`: (optional) Number of moves in the game
- `moves`: Array of move records
  - `OpeningRemoval`: Initial piece removal with color and position
  - `Jump`: A jump with from/to positions; `removed` lists the positions of the removed stones
