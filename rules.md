# Rules and Game Description

Kōnane is a traditional Hawaiian strategy board game. The proper spelling of the name of the game is "Kōnane".

## Board and Pieces Specification

* **The Board (Papamū):** * A rectangular grid of holes (*lua*).
* Standard sizes are square with an even-number of lua on each side.

* **The Pieces (ʻIliʻili):** * Two colors: **Black** (lava stone) and **White** (coral).
* Initial State: Every cell on the board is filled.
* Pattern: Pieces must be placed in an alternating "checkerboard" pattern.
* *Note: In a correct setup, all diagonal lines consist of stones of the same color.*

## Game Phases and Initialization

### Phase 1: Setup

1. Initialize the board with alternating Black and White pieces such that the first lua contains a Black piece.
2. Assign players to "Black" and "White." Black always takes the first move.

### Phase 2: The Opening (Removal)

The game begins with two specific removals before any jumping occurs:

1. **Black's First Move:** The Black player must remove one Black stone from the board. This removal is restricted to:
   * The **Center** of the board.
   * The **Corners** of the board.
2. **White's First Move:** The White player must remove one White stone that is **orthogonally adjacent** to the empty lua created by Black.
3. **Result:** The board now has exactly two empty adjacent empty lua.

## Movement and Capture Logic (The Kuʻi)

After the Opening Phase, all subsequent moves must follow these strict capturing rules:

* **Turn-Based:** Players alternate turns.
* **Move Type:** All moves must be **capturing jumps**. There are no non-capturing moves.
* **Direction:** Jumps must be **orthogonal** (Up, Down, Left, Right). No diagonal jumps.
* **Mechanic:** A player moves their stone over an opponent's stone into an empty lua immediately behind it.
* **Removal:** The jumped opponent's stone is removed from the board.
* **Multi-Jumps:** * A player may perform multiple captures in a single turn using the same stone. During a multi-jump, the player **cannot change direction**. The stone must continue in the same straight line (e.g., if the first jump was "Up," all subsequent jumps in that turn must be "Up").
* **Optionality:** Multi-jumps are not mandatory. A player may choose to stop after any capture, even if further captures are possible.

## Win/Loss Condition

Kōnane uses a "Last Player to Move" victory condition (normal play convention):

* **Termination:** If a player cannot make a legal jump on their turn, the game ends immediately.
* **Winner:** The player who made the **last successful move** is the winner.
* **Loser:** The player who is unable to move is the loser.

## Algebraic Notation

### 1. The Coordinate System

The board uses a standard grid system:

* **Columns (Files):** Labeled with lowercase letters from left to right (e.g., **a, b, c, d...**).
* **Rows (Ranks):** Labeled with numbers from bottom to top (e.g., **1, 2, 3, 4...**).
* **Origin:** The bottom-left corner is always **a1**.

### 2. Opening Phase Notation

The initial removals are recorded by the coordinate of the stone removed:

* **Black's Removal:** Recorded as the coordinate alone (e.g., `e4`).
* **White's Removal:** Recorded as the coordinate alone (e.g., `e5`).

### 3. Movement and Capture Notation

Captures are recorded using the **Starting Square**, a **dash**, and the **Ending Square**.

* **Single Jump:** If a piece on `c3` jumps over a piece on `d3` to land on `e3`, it is written as: **`c3-e3`**.
* **Multi-Jump:** Because multi-jumps must continue in a straight line, you only need to record the final destination. If a piece on `c3` jumps three pieces in a row to land on `i3`, it is written as: **`c3-i3`**.
* **Ambiguity:** Note that the intermediate jumped pieces are implied because the direction must be a straight line.

### 4. Game Log Example

A typical game transcript looks like this:

| Turn | Move |
| --- | --- |
| 1 | **e4**    |
| 2 | **d4**    |
| 3 | **f4-d4** |
| 4 | **d3-d5** |

### 5. **Result Codes:**

* `1-0`: Black Wins
* `0-1`: White Wins
* `1/2-1/2`: Draw (Highly rare in Kōnane, but theoretically possible in some rule variations).
