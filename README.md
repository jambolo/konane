# Kōnane

![Rust Workflow](https://github.com/jambolo/game-player/actions/workflows/rust.yml/badge.svg)

Kōnane is a traditional Hawaiian strategy board game.

## Game Description

- Played on a rectangular grid (typically square, even-sized).
- Two players: Black (lava stone) and White (coral).
- Board is filled in a checkerboard pattern; all cells occupied at start.

## Rules

### Setup & Opening

1. Board is initialized with alternating Black and White pieces; a1 (bottom-left) is always Black.
2. Black removes one Black stone from the center or a corner.
3. White removes one White stone orthogonally adjacent to the empty space.

### Movement & Captures

- Players alternate turns.
- All moves are capturing jumps; no non-capturing moves.
- Jumps are orthogonal only (up, down, left, right).
- Jump over an opponent’s stone into an empty space; remove the jumped stone.
- Multi-jumps allowed in a straight line (no direction change); optional to continue.

### Win Condition

- If a player cannot make a legal jump, the game ends.
- The last player to make a move wins.

For full rules and examples, see [rules.md](rules.md).

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
    { "Jump": { "color": "Black", "from": { "row": 3, "col": 5 }, "to": { "row": 3, "col": 3 }, "captured": [{ "row": 3, "col": 4 }] } }
  ]
}
```

- `board_size`: Board dimension (4-16, must be even)
- `winner`: (optional) "Black" or "White" - only present for completed games
- `total_moves`: (optional) Number of moves in the game
- `moves`: Array of move records
  - `OpeningRemoval`: Initial piece removal with color and position
  - `Jump`: Capturing move with from/to positions and captured piece positions

Import reads from `konane_game.json` in the current directory. Incomplete games can be imported and continued from where they left off.
