# Development

How Kōnane is built, tested, and put together. The rules of the game are in [rules.md](rules.md), which is the authoritative reference for any rules question. The JSON game format is described in [README.md](README.md#json-game-format).

## Prerequisites

- Rust stable with edition 2024 support (1.85 or later). CI uses the current stable toolchain; no toolchain is pinned.
- Network access for the first build. `game-player` is a git dependency pinned to a tag in [Cargo.toml](Cargo.toml).
- No system packages beyond the Rust toolchain. CI builds on `ubuntu-latest` and `windows-latest` without installing any.

## Build and run

```sh
cargo build
cargo run --release
cargo run --release -- --ai-depth 6
```

- `--ai-depth` sets the AI's search depth in plies. The default is 8.
- Use `--release` when playing against the AI. The search is CPU-bound.
- Dependencies are compiled with optimizations in debug builds too (`[profile.dev.package."*"]` in [Cargo.toml](Cargo.toml)). The first debug build is slower for it. Without it, a debug build takes about 0.4 s to draw each frame when iced uses its software renderer.
- iced falls back to its software renderer when it cannot create a GPU device, as under WSL2. Set `ICED_BACKEND=wgpu` to make it fail with the reason instead of falling back.
- The stone and background images, the fonts, and the window icon are compiled into the binary with `include_bytes!`, so it runs from any directory. [data/](data/) holds many image variants; the ones in use are named at the top of [src/ui/board_view.rs](src/ui/board_view.rs). The fonts are in [data/fonts/](data/fonts/) with their licenses.
- The look and wording follow the brand guide in [docs/branding/](docs/branding/README.md). Read it before changing anything a player sees.

### Features

| Feature | Default | Effect |
| --- | --- | --- |
| `random-openings` | on | The AI picks each opening removal at random instead of searching every candidate. The opening gives the AI nothing it can use, so searching it only costs time. |

## Test

```sh
cargo test --workspace
cargo test --workspace --no-default-features
```

- The second command covers the exhaustive-opening code path, which is compiled only when `random-openings` is off. CI does not build that configuration, so run it locally after touching [src/game/ai.rs](src/game/ai.rs).
- Unit tests live in a `#[cfg(test)]` module at the bottom of each source file.
- [tests/import_tests.rs](tests/import_tests.rs) tests game import through the library's public API.
- The tests in [src/ui/app.rs](src/ui/app.rs) drive `KonaneApp::update` with messages and never open a window. Add UI behavior tests there.
- `bitboard_equivalence` in [src/game/rules.rs](src/game/rules.rs) plays random games and compares the bitboard move generator against a lua-by-lua reference. Keep it passing after any change to move generation.

## Before opening a PR

Run what CI runs. All four must pass.

```sh
cargo build --workspace --all-targets
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

- Formatting uses [.rustfmt.toml](.rustfmt.toml): `max_width = 132`.
- Clippy warnings are errors.
- The format and clippy job runs only on pull requests. Build and test run on Ubuntu and Windows.
- Add or update tests for any change to rules, move generation, import, or undo/redo.
- Update [README.md](README.md) if the JSON format or user-visible behavior changes, and this file if the architecture changes.

## Branches and releases

- `develop` is the integration branch. Work happens on `feature/*` branches off it.
- `master` holds releases.
- CI ([.github/workflows/ci.yml](.github/workflows/ci.yml)) runs on pushes to `master`, `develop`, and `release/**`, and on every pull request. Pushes to `develop` also upload coverage to Codecov using `cargo llvm-cov`.
- CD ([.github/workflows/cd.yml](.github/workflows/cd.yml)) runs when a push to `master` changes `Cargo.toml`. If no tag `v<version>` exists for the package version, it creates one and merges `master` back into `develop`. To release, bump `version` in `Cargo.toml` and merge to `master`.

## Architecture

### Crate layout

The package is a library plus a binary.

- The library (`konane`, [src/lib.rs](src/lib.rs)) contains `game` and `import`. Nothing in it uses iced.
- The binary ([src/main.rs](src/main.rs)) contains `ui` and parses the command line. It imports `konane::game` at the crate root, so UI code refers to it as `crate::game`.

The game, rules, and UI are kept apart for these reasons:

- Game state can be serialized and deserialized independently.
- Rules can be unit tested without a UI.
- Different UIs (CLI, web) could use the same game and rules modules.
- AI players can query game state and rules without touching the UI.

### Modules

| Module | Contents |
| --- | --- |
| [src/game/state.rs](src/game/state.rs) | `PieceColor`, `Position`, `Direction`, `GamePhase`, `MoveRecord`, `Board`, `KonaneState`, and the `MoveHistory` and `UndoRedoStack` aliases |
| [src/game/bitboard.rs](src/game/bitboard.rs) | `Bitboard`, a set of lua with shift and set operations |
| [src/game/rules.rs](src/game/rules.rs) | `Jump` and `Rules`: legal opening removals, jump generation, applying moves, detecting the end of the game |
| [src/game/zhash.rs](src/game/zhash.rs) | `ZHash`, the incremental Zobrist hash of a game state |
| [src/game/ai.rs](src/game/ai.rs) | `AiPlayer` and the adapters that plug Kōnane into the `game-player` minimax search |
| [src/game/player.rs](src/game/player.rs) | `Player` trait, `PlayerMove`, and `HumanPlayer` |
| [src/import.rs](src/import.rs) | JSON game import with full move validation |
| [src/ui/app.rs](src/ui/app.rs) | `KonaneApp`: top-level state, message handling, undo/redo, AI scheduling |
| [src/ui/theme.rs](src/ui/theme.rs) | Brand colors, fonts, widget styles, the background pattern and the mark |
| [src/ui/board_view.rs](src/ui/board_view.rs) | Board rendering, selection, removal animations |
| [src/ui/setup_view.rs](src/ui/setup_view.rs) | Setup screen and the import dialog |
| [src/ui/game_over_view.rs](src/ui/game_over_view.rs) | Game-over overlay and the export dialog |

### Game state

`KonaneState` holds the board, the phase, the player to move, the position of Black's opening removal, and the Zobrist hash.

- Phases run `OpeningBlackRemoval` → `OpeningWhiteRemoval` → `Play` → `GameOver { winner }`.
- Every mutator on `KonaneState` updates the hash incrementally. Change state only through those methods, or the hash goes stale and the AI's transposition table returns wrong entries.
- `Rules` is stateless. `Rules::apply_opening_removal` and `Rules::apply_jump` are the only functions that advance a game. Each returns the `MoveRecord` for the move, and each sets `GameOver` when the next player has no jump.

### Board representation

`Board` stores only an occupancy `Bitboard`, which is one `u16` per rank.

- Jumps move two lua, so a stone never changes lua parity. Its color is implied by `(row + col) % 2` and is not stored.
- The 16×16 maximum comes from the `u16` rank. Raising it means widening `Bitboard`, changing `MAX_SIZE` in both `bitboard.rs` and `zhash.rs`, and updating the size checks in `Board::new`, `import.rs`, and the setup view.

### Move generation

Move generation in `rules.rs` shifts whole sets of stones at once rather than visiting each lua.

- Landings for one jump in direction `d` are `shift(shift(own, d) & opponents, d) & vacancies`.
- Multi-jumps repeat that step on the unmodified board, feeding each set of landings back in as the movers.
- `Jump` is `{from, direction, length}`, where `length` is the number of stones removed. `to()` and `removed()` are derived from it.

### AI

The search itself lives in the [game-player](https://github.com/jambolo/game-player) crate. `ai.rs` supplies what it needs:

- `State for KonaneState`: Black is `Alice`, White is `Bob`. `apply` clones the state and applies one action.
- `KonaneEvaluator`: the score is Black's mobility minus White's, where mobility is the number of available jumps. A win is ±1000. Opening phases score 0.
- `KonaneMoveGenerator`: produces opening removals or every legal jump, depending on the phase.

The Zobrist hash is the transposition-table key. Its tables are generated with a fixed-seed `ChaCha8` RNG, so hashes are the same on every run.

### UI

The UI is an [iced](https://iced.rs) application: `KonaneApp::update` handles a `Message` and `KonaneApp::view` builds the widget tree.

- **Views.** `AppView` is `Setup`, `Playing`, or `GameOver`. The game-over view is the playing view with an overlay stacked on top.
- **AI moves.** `maybe_trigger_ai_move` runs the search on a blocking tokio thread and delivers the result as `Message::AiMoveComputed`. A global lock runs searches one at a time. `ai_generation` is incremented whenever a pending result must be discarded (new game, undo, redo), and a result carrying an old generation is ignored. A search that has already started is not interrupted.
- **Undo and redo.** Before each move, a snapshot of `(KonaneState, MoveHistory)` is pushed onto the undo stack. Undo returns to the latest snapshot in which a human is to move, so the AI does not immediately replay the move that was undone. Redo steps forward to the next human turn.
- **Board rendering.** The board is three canvases in a `Stack`: background, highlights, stones. Each has its own cache. They are separate canvases because iced draws images above paths within a single frame, so layering needs separate widgets.
- **Animations.** A removed stone fades out over 300 ms. Fading stones are drawn outside the stone cache, and the stone canvas requests redraws while any fade is running.
- **Theme.** Every color, font, and widget style comes from `theme.rs`. Views do not define colors of their own. The fonts are loaded in `main.rs`.
- **Human input.** The UI handles clicks directly. `HumanPlayer` in `player.rs` is not used by it.

### Import and export

- Import replays the JSON move list through `Rules`, rejecting the file at the first illegal move, and returns the final state, the move history, and an undo stack with one snapshot per move. An imported game can therefore be undone back to the start.
- Export is offered when a game ends, as text or JSON. The text log is one numbered move per line followed by a result code.

## Conventions

- **Terminology.** A spot on the board is a *lua* (plural *lua*). Use "position" for coordinates. A jumped stone is *removed*, never "captured": it is not kept or counted and plays no further part in the game.
- **Coordinates.** `Position(row, col)` follows algebraic notation. `(0, 0)` is the bottom-left corner, a1. Rows increase upward (rank 1, 2, 3…) and columns increase rightward (file a, b, c…).
- **Board sizes.** 4×4 to 16×16, even sizes only. The setup view enforces this. Rules and AI code assume an even-sided square board.
- **Initial placement.** Checkerboard, with Black where `(row + col) % 2 == 0`. a1 is always Black.
- **Move notation.** An opening removal is written as its position (`d4`). A jump is written `from-to` (`f4-d4`).
- **Result codes.** `1-0` means Black wins and `0-1` means White wins. Black is listed first, the opposite of the chess convention.
- **Comments.** Keep them to brief descriptions of what the code does.

## Known limitations

- No network play. It would need moves serialized over a socket.
- Black always makes the first opening removal. The `first_player` argument to `KonaneState::new` is ignored.
- Only finished games can be exported.
