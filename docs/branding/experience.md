# Easy and fun

Kōnane is for casual computer users and casual players. If a choice makes the app cleverer but harder, it is the wrong choice.

## Easy

- **One click to play.** The setup screen opens with choices that work: an 8×8 board, you as Black, the computer as White. Play is the only turmeric button on the screen.
- **Always show what is possible.** Stones that can move and lua they can land in are marked. A player should never have to guess whether a move is legal.
- **Always say what to do next.** The status line is an instruction in plain words, addressed to the player whose turn it is.
- **Nothing is final.** Undo is always visible during a game and never asks "are you sure".
- **No typing.** Saving and opening a game use the system's file dialog. Nobody types a path.
- **No technical words on screen.** No JSON, AI, depth, ply, import, export, or algebraic notation. See [voice.md](voice.md#english-terms).
- **Strength in plain words.** The computer's level is Easy, Medium, or Hard. Search depth stays a command-line option for people who want it.
- **Teach in the app.** "How to play" is reachable from the setup screen and fits on one screen: the opening, the jump, how the game ends.
- **Comfortable to read.** Body text is at least 16 px. Buttons are at least 36 px tall. Contrast follows [visual.md](visual.md#contrast).
- **Small boards first.** Offer 6×6 as a quick game. Large boards are available and not pushed.

## Fun

- **Stones feel like stones.** They are photographs of lava and coral, each a little different. A stone that lands makes the sound of stone on stone. Sound has an obvious off switch.
- **Removing stones is satisfying.** A jumped stone fades out over 300 ms. A multi-jump plays as a quick run, one stone after another.
- **A win gets a moment.** A full moon rises behind the result, because *kōnane* also means bright moonlight. It lasts about a second and never blocks the buttons.
- **A good loser and a good winner.** The computer never gloats. The result always says why the game ended: "White has no jump left."
- **Play again in one click.** The result screen offers New game first.

## Not fun

- No scores, streaks, ranks, timers, or daily goals.
- No pop-ups that interrupt a game.
- No mascots, confetti, or cartoon sounds.
- No sign-in, ads, or prompts to rate the app.

## In the app

Where the app differs from this guide today.

| Element | Location | Current | Brand |
| --- | --- | --- | --- |
| Opening a saved game | [src/ui/setup_view.rs](../../src/ui/setup_view.rs) | Typed file path | System file dialog |
| Saving a game | [src/ui/game_over_view.rs](../../src/ui/game_over_view.rs) | Typed file path, Text or JSON | System file dialog, one "Save this game" button |
| Computer strength | [src/main.rs](../../src/main.rs) | `--ai-depth` only | Easy, Medium, Hard on the setup screen |
| How to play | | None in the app | One screen, linked from setup |
| Sound | | None | Stone on stone, with an off switch |
| Win moment | [src/ui/game_over_view.rs](../../src/ui/game_over_view.rs) | "Game Over!" panel | Moonrise, result, and the reason |
