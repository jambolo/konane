# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo run                      # Run the game (AI depth: 8)
cargo run -- --ai-depth 12     # Run with custom AI depth
cargo test                     # Run all tests
cargo test <test_name>         # Run specific test
cargo clippy                   # Lint
cargo fmt                      # Format code
```

### Version Bumping

1. Read version from `Cargo.toml`, compute next minor version.
2. Create release branch: `git checkout -b release/<new_version>`.
3. Update version in `Cargo.toml` and verify build: `cargo build --release`. Verify tests: `cargo test --all`
4. Commit all changed files.
5. Tag: `git tag v<new_version>`.
6. Merge to master: `git checkout master && git merge --no-ff release/<new_version>`. Push: `git push origin master`. Push tag: `git push origin master --tags`
7. Merge to develop: `git checkout develop && git merge --no-ff master`. Push: `git push origin develop`.

## Kōnane Design

### Architecture

**Module Structure:**

- `game/state.rs`: Pure data structures (Board, GameState, Position, PieceColor, MoveRecord)
- `game/rules.rs`: Game logic (move validation, jump calculation, state transitions)
- `game/player.rs`: Player abstraction trait
- `game/ai.rs`: AI player using minimax search
- `game-player/`: Submodule providing minimax with alpha-beta pruning and transposition table
- `ui/`: All iced UI code, decoupled from game logic

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

### Initial Stone Placement

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

`Player` trait in `game/player.rs`:

- `request_move()` - AI returns computed move, human returns None
- `receive_input()` - UI sends clicks to human player
- `is_ready()` - Check if player has pending move

### AI Player

`AiPlayer` in `game/ai.rs` implements the `Player` trait using the `game-player` minimax search.

**Integration with game-player submodule:**

- `KonaneState`: Wrapper implementing `State` trait (fingerprint, whose_turn, is_terminal, apply)
- `KonaneEvaluator`: Implements `StaticEvaluator` using mobility heuristic
- `KonaneMoveGenerator`: Implements `ResponseGenerator` for opening removals and jumps
- Transposition table with 100k entries for position caching
- Search depth configurable via `--ai-depth` command-line argument (default: 8)

**UI integration:**

- Setup view allows selecting Human or AI for each player
- AI moves computed asynchronously via `Task::perform`
- `ai_computing` flag prevents input during AI turns

### Animations

- Removal animations: pieces fade out and shrink over 300ms
- Animation system uses iced subscriptions for frame updates
- `RemovalAnimation` tracks position, color, and start time

### Undo/Redo

- State history maintained via `undo_stack` and `redo_stack` in `KonaneApp`
- `save_state_for_undo()` clones current state before each move
- Undo/Redo buttons in info bar during play
- Imported games populate undo stack with full history

### Game Import/Export

- `import.rs`: Validates and replays JSON game files
- Import modal in setup view
- Export modal in game over view (Text or JSON format)
- JSON format: `{ board_size, winner?, moves[] }`
- Text format: Algebraic notation with result code

### Board Display

- Row and column labels (algebraic notation: a-p, 1-16)
- Stone images from `data/` directory (`black-stone-15.png`, `white-stone-15.png`)
- Board scales to fit window
- Move history panel shows game record in algebraic notation

### Known Limitations / Future Work

1. **No network play** - Would need to serialize moves over socket

## Dependencies

- iced 0.14 (canvas, image, tokio features)
- ndarray for board state
- serde/serde_json for serialization
- game-player submodule (minimax search)
- Rust 2024 edition
