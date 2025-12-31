# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## General Instructions for Claude

### Token Discipline

- Be concise by default.
- No explanations unless explicitly requested.
- No restating the question.
- No summaries at the end.
- Use bullet points only when clarity improves.
- Prefer short sentences.
- Assume reader is expert.

### Output Rules

- Answer the question directly.
- Do not add context, background, or alternatives unless asked.
- If uncertain, say "unknown" or ask one clarifying question.

### Code

- Output code only, no commentary.
- Prefer minimal, idiomatic solutions.
- Limit comments to very brief descriptions of what the code does. Do not describe why changes were made.

### Interaction

- Ask at most one clarifying question.
- Never suggest next steps unless requested.

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo build --features audio   # Build with audio support
cargo run                      # Run the game
cargo run --features audio     # Run with sound effects
cargo test                     # Run all tests
cargo test <test_name>         # Run specific test
cargo clippy                   # Lint
cargo fmt                      # Format code
```

## Version Bumping

1. Read version from `Cargo.toml`, compute next minor version.
2. Create release branch: `git checkout -b release/<new_version>`.
3. Update version in `Cargo.toml` and commit.
4. Verify build: `cargo build --release`.
5. Merge to master: `git checkout master && git merge release/<new_version>`. Tag: `git tag v<new_version>`. Push: `git push origin master --tags`.
6. Merge to develop: `git checkout develop && git merge master`. Push: `git push origin develop`.

## Kōnane Design

### Architecture

**Module Structure:**

- `game/state.rs`: Pure data structures (Board, GameState, Position, PieceColor, MoveRecord)
- `game/rules.rs`: Game logic (move validation, jump calculation, state transitions)
- `game/player.rs`: Player abstraction trait for future AI/network players
- `ui/`: All iced UI code, decoupled from game logic
- `audio.rs`: Kira-based sound effects (optional feature)

**Why this separation:**

- Game state can be serialized/deserialized independently
- Rules can be unit tested without UI
- Different UIs (CLI, web) could use the same game/rules modules
- AI players can query game state and rules without touching UI

### UI Framework: iced

- Pure Rust, no FFI
- Elm-like architecture (Model-View-Update) fits game state management
- Canvas widget for custom board rendering
- Cross-platform

### Game Phases (GamePhase enum)

1. `Setup` - Not used directly; UI handles pre-game configuration
2. `OpeningBlackRemoval` - Black removes one piece from center/corner
3. `OpeningWhiteRemoval` - White removes adjacent piece
4. `Play` - Normal capturing jumps
5. `GameOver { winner }` - Winner determined

### Multi-Jump Implementation

Per rules: multi-jumps must continue in same direction.

- `valid_jumps_from()` returns all possible endpoints (single and multi-jump)
- Each `Jump` struct contains the full path and all captured positions
- Player selects destination; captures computed from jump data

### Coordinate System

Uses algebraic notation conventions matching the rules specification:

- Position(row, col) where (0,0) is the **bottom-left corner** (a1)
- Row increases **upward** (rank 1, 2, 3...)
- Column increases **rightward** (file a, b, c...)
- `Position::to_algebraic()` converts to "a1", "e4" format
- `Position::from_algebraic()` parses algebraic notation

### Color Placement

- Checkerboard pattern: `(row + col) % 2 == 0` is Black
- Position a1 (0,0) is always Black per rules ("first lua contains a Black piece")
- **Corner colors on even-sized boards:**
  - a1 (0,0) = Black (sum 0, even)
  - a8 (7,0) on 8x8 = White (sum 7, odd)
  - h1 (0,7) on 8x8 = White (sum 7, odd)
  - h8 (7,7) on 8x8 = Black (sum 14, even)

### Move Logging

- `MoveRecord` enum: `OpeningRemoval` or `Jump`
- Text output: strict algebraic notation (e.g., "1. e4", "2. d4", "3. f4-d4")
- Result codes: `1-0` (Black wins), `0-1` (White wins)
- JSON export available via serde

### Player Abstraction

`Player` trait defined but not yet integrated:

- `request_move()` - AI returns computed move, human returns None
- `receive_input()` - UI sends clicks to human player
- `is_ready()` - Check if player has pending move

**Current state:** Human input handled directly in UI. Trait ready for AI integration.

### Audio

- Uses Kira audio library for sound effects (optional feature)
- Move sound: 440Hz click on piece movement
- Capture sound: 330Hz click on piece removal
- Sounds generated programmatically as WAV data

### Animations

- Removal animations: pieces fade out and shrink over 300ms
- Animation system uses iced subscriptions for frame updates
- `RemovalAnimation` tracks position, color, and start time

### Known Limitations / Future Work

1. **No undo** - Would need state history stack
2. **No save/load** - GameState is serializable, just need file dialog
3. **No AI player** - Trait defined, needs minimax/alpha-beta implementation
4. **No network play** - Would need to serialize moves over socket
5. **Simple graphics** - Canvas shapes only; image tiles not yet implemented

## Dependencies

- iced 0.14 (canvas, image, tokio features)
- ndarray for board state
- kira 0.11 for audio (optional)
- serde/serde_json for serialization
- Rust 2024 edition
