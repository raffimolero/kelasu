# Kelasu

-   Author: **MaxTheFox**
-   Original rules template: **KittyTac**
-   Implementation: **Redstoneboi** (raffimolero)
-   Version: **0.1**

> "This is an abstract game for my setting."

\- MaxTheFox

## Components

-   1 10x10 Board
-   2x Piece Set (**Red** and **Blue**)
-   In each set:
    -   20x Blank
    -   10x Warrior
    -   5x Runner
    -   5x Diplomat
    -   4x Champion
    -   2x General
    -   4x Stone

## Setup

The board, with the 4 victory squares located in the middle, is set up like this:

```
B B B B B B B B B B
B B B B B B B B B B
S   S         S   S

        : :
        : :

s   s         s   s
b b b b b b b b b b
b b b b b b b b b b
```

Where:

|  Character  | Tile                 |
| :---------: | :------------------- |
|     `B`     | Blue Blank           |
|     `S`     | Blue Stone           |
|     `b`     | Red Blank            |
|     `s`     | Red Stone            |
|     `:`     | Empty Victory Square |
| `UPPERCASE` | Blue                 |
| `lowercase` | Red                  |

MOVEMENT AND CAPTURE
Players take turns moving, as in other abstract games. Blue goes first.
The number of pieces that can take part in a move is determined by the number of stones their player has - at the start it's 4.
The different pieces move in different ways, and capture as in chess.

| Icon | Meaning      |
| :--: | :----------- |
| `.`  | Move Only    |
| `X`  | Move/Capture |
| `*`  | Must Capture |
| `%`  | Convert      |

```
Blank:




          .
        . b .





Warrior:




        * X *
        X w X





Runner: Move only for 1st
X
  X               X
    X           X
      X       X
        .   .
          r
        .   .
      X       X
    X           X
  X               X

Diplomat:


          .
          .
        % . %
    . . . d . . .
        % . %
          .
          .


Champion: Infinite Move/Capture forward, Infinite Move back
          X
          X
          X
          X
        X X X
    X X X c X X X
          .
          .
          .
          .

General: Infinite Move/Capture in all directions
X         X
  X       X       X
    X     X     X
      X   X   X
        X X X
X X X X X g X X X X
        X X X
      X   X   X
    X     X     X
  X       X       X

Stone:





          s





```

Converting a piece sacrifices the original Diplomat.
Warriors can Recall to the first row of their side, on the same file, if they reach the end of the board.

MERGING
Notice how there are only blanks and stones on the board at the start.
Blanks that are next to each other and outside of their starting 2 rows can merge into another piece on any of the blanks' squares.
This counts as a move for every blank involved, but if you do not have enough stones it simply counts as the maximum amount of moves you have. The costs are as follows:

# TODO: MAKE TABLE

|Piece|Cost|
Warrior = 2 blanks
Runner = 4 blanks
Diplomat = 4 blanks
Champion = 5 blanks
General = 10 blanks
Stone = 21 blanks (:p)

### Win Conditions

The game is won by a player if:

1. The opponent resigns.
2. The opponent has no stones left.
3. The opponent has no non-stone pieces left.
4. The player has 4 of their pieces in the 4 central squares.

### Draw Conditions

1. 64-move rule, as in chess. The things that reset the counter are a blank move, a merging, or a capture.
2. Fourfold repetition of the position.
3. Both players being unable to achieve the win conditions. (not implemented)
4. Stalemate, though this is extremely rare and achieving it probably requires breaking YLK rule 16.3.1 "Bringing the game into disrepute".
