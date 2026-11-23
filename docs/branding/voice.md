# Voice and language

## Voice

Kōnane talks like a friend who knows the game well and is glad to show you. It is plain, brief, and good-humored. The reader is a casual computer user, so no sentence should need a second reading.

| Principle | Do | Don't |
| --- | --- | --- |
| Say what to do next | "Choose a stone." | "It is now your turn to select a piece." |
| Teach in passing | "White has no jump left." | "Game Over!" |
| Name the real thing | "Remove one stone next to the empty lua." | "Remove a piece adjacent to the empty space." |
| Enjoy the game | "Three stones in one jump." | "Multi-jump executed." |
| Stay level | "Black wins." | "Black wins!!!" |
| Skip the computer words | "Save this game" | "Export JSON to file path" |
| Own the error | "That file isn't a saved kōnane game." | "Error: invalid input" |

- Address the player as "you". Address a color by name when two people share a screen: "Black, choose a stone."
- Sentence case everywhere, including buttons and titles.
- No exclamation marks in status lines or errors. One is allowed on a win.
- Use the ellipsis character (…), not three periods.
- An error says what happened and what to do about it.

## Hawaiian spelling

These rules apply to every string, document, and image.

- **Write the ʻokina and the kahakō, always.** They are letters and marks of the language, not decoration. *Kōnane* without the kahakō is misspelled.
- **The ʻokina is U+02BB (ʻ).** It is not an apostrophe (U+0027), a left quote (U+2018), or a backtick.
- **The kahakō vowels are ā ē ī ō ū** and Ā Ē Ī Ō Ū.
- **No English plural -s.** One lua, two lua. One ʻiliʻili, many ʻiliʻili.
- **Capitalize the letter after an ʻokina.** ʻIliʻili at the start of a sentence, not ʻiliʻili.
- **No italics for Hawaiian words.** They are not foreign in Hawaiʻi. This follows the University of Hawaiʻi style guidance.
- **Explain once.** On first use in a screen or page, give the meaning in a few plain words: "ʻiliʻili, the stones". After that, use the word alone.
- **Hawaiʻi** takes an ʻokina. **Hawaiian** is an English word and does not.
- If a typeface cannot draw these characters, change the typeface. Never drop the marks. See [visual.md](visual.md#typography).

## Which Hawaiian words to use

Use Hawaiian for the game's own things. Do not add Hawaiian words for atmosphere.

| Word | Meaning | Use |
| --- | --- | --- |
| kōnane | The game. Also: bright moonlight. | The game and, capitalized, the app |
| papamū | The stone on which kōnane is played | The board, in descriptive copy. "Board" is fine in controls. |
| ʻiliʻili | Pebble, small stone | The stones, in descriptive copy. "Stone" is fine in instructions. |
| lua | Hole, pit | One hollow on the board. Never "square", "cell", or "space". |
| ʻeleʻele | Black | Only when describing the stones: ʻiliʻili ʻeleʻele |
| keʻokeʻo | White | Only when describing the stones |

Meanings are from Pukui and Elbert, *Hawaiian Dictionary* (1986), searchable at [wehe.hilo.hawaii.edu](https://wehe.hilo.hawaii.edu/).

Words such as aloha and mahalo are not part of the brand's vocabulary. They carry real meaning and are not ours to use as seasoning.

## English terms

| Use | Not |
| --- | --- |
| stone | piece, checker, token, pebble |
| Black, White (the players) | dark, light, player 1, player 2 |
| lava stone, coral (the materials) | black piece, white piece |
| jump (the move) | capture, take, hop |
| remove (a stone, in the opening or by a jump) | capture, take, delete, discard |
| computer | AI, bot, engine |
| person | human |
| saved game | game log, export, JSON |

### Remove, not capture

A jumped stone is removed. It is not captured: nobody keeps it or counts it, and it plays no further part in the game. Every move is a jump that removes at least one stone, so "remove" is also the word for the opening.

- "Black removed two stones.", "removed stones", "the stone you jumped is removed".
- Never "capture", "captured", or "a capture" in anything a player reads.

## Screen wording

Current text in the app and the wording that fits this guide.

| Where | Current | Brand |
| --- | --- | --- |
| Setup subtitle | Traditional Hawaiian Board Game | The Hawaiian game of lava stone and coral |
| Setup | Board Size: | Board size |
| Setup | Black Player: / White Player: | Black / White |
| Setup | Human / AI | Person / Computer |
| Setup | Start Game | Play |
| Setup | Import Game | Open a saved game |
| Status, opening | Black: Remove a black piece from the center or a corner | Black, remove one of your stones from the center or a corner. |
| Status, opening | White: Remove a white piece adjacent to the empty space | White, remove one of your stones next to the empty lua. |
| Status, play | Black's turn - Select a piece to move | Black, choose a stone. |
| Status, play | Black's turn - Select destination | Choose where to land. |
| Status, computer | Black (AI) is thinking... | Black is thinking… |
| Side panel | Current: Black | Black to move |
| Game over | Game Over! / Black wins! | Black wins / White has no jump left. |
| Game over | Total moves: 42 | 42 moves |
| Game over | Download game log: | Save this game |
| Everywhere | New Game | New game |
| Dialogs | Error: {message} | What happened, then what to do. "That file isn't a saved kōnane game. Choose a .json file saved from Kōnane." |

## Describing the game

Short:

> Kōnane is a Hawaiian game for two. Fill the board with lava stone and coral, take one of each away, then jump until someone can't.

Longer:

> Kōnane is a Hawaiian strategy game played with black lava stones and white coral on a papamū, a board of shallow lua. Every move is a jump over one of your opponent's stones. The last player able to jump wins.
