# Kelasu

- Author: **MaxTheFox**
- [Original rules template](https://pastebin.com/cCzZXVAw): **KittyTac**
- Implementation and README: **Redstoneboi** (it me)
- Discord Server management: **CCCFan**
- Version: **0.1**

> "This is an abstract game for my setting."

\- MaxTheFox

[Discord Link](https://discord.gg/3jzTb6xbTM) (dead lol)

# Board

An empty board, with the 4 Victory Tiles located in the middle, looks like this:

```hs
   ╔[0-1-2-3-4-5-6-7-8-9]╗
 A ║                     ║ A
 B ║                     ║ B
 C ║                     ║ C
 D ║                     ║ D
 E ║         : :         ║ E
 F ║         : :         ║ F
 G ║                     ║ G
 H ║                     ║ H
 I ║                     ║ I
 J ║                     ║ J
   ╚[0-1-2-3-4-5-6-7-8-9]╝
```

It is a 10 by 10 board, where **Ranks** are labeled with **letters** and **Files** are labeled with **numbers**, unlike Chess.

This is the `D`-Rank:

```hs
   ╔[0-1-2-3-4-5-6-7-8-9]╗
 A ║                     ║ A
 B ║                     ║ B
 C ║                     ║ C
"D=║-.-.-.-.-.-.-.-.-.-.-║=D"
 E ║         : :         ║ E
 F ║         : :         ║ F
 G ║                     ║ G
 H ║                     ║ H
 I ║                     ║ I
 J ║                     ║ J
   ╚[0-1-2-3-4-5-6-7-8-9]╝
```

This is `D2`:

```hs
   ╔[0-1-2-3-4-5-6-7-8-9]╗
 A ║                     ║ A
 B ║                     ║ B
 C ║                     ║ C
 D ║    ( )              ║ D
 E ║         : :         ║ E
 F ║         : :         ║ F
 G ║                     ║ G
 H ║                     ║ H
 I ║                     ║ I
 J ║                     ║ J
   ╚[0-1-2-3-4-5-6-7-8-9]╝
```

# Pieces

The different pieces move in different ways, and capture as in chess.

|    Icon     | Meaning        |
| :---------: | :------------- |
| `UPPERCASE` | **Blue** Piece |
| `lowercase` | **Red** Piece  |
|     `.`     | Move Only      |
|     `#`     | Move/Capture   |
|     `*`     | Must Capture   |
|     `%`     | Convert        |
|     `@`     | Recall         |

```hs
                        Blanks
                ╔[0-1-2-3-4-5-6-7-8-9]╗
              A ║                     ║ A
              B ║                     ║ B
              C ║     . B .           ║ C
              D ║       .             ║ D
              E ║                     ║ E
              F ║                     ║ F
              G ║             .       ║ G
              H ║           . b .     ║ H
              I ║                     ║ I
              J ║                     ║ J
                ╚[0-1-2-3-4-5-6-7-8-9]╝

-------------------------------------------------------

          Warriors              Red Warrior Recall
   ╔[0-1-2-3-4-5-6-7-8-9]╗   ╔[0-1-2-3-4-5-6-7-8-9]╗
 A ║                     ║ A ║         # w #       ║ A
 B ║                     ║ B ║                     ║ B
 C ║     # W #           ║ C ║                     ║ C
 D ║     + # +           ║ D ║                     ║ D
 E ║                     ║ E ║                     ║ E
 F ║                     ║ F ║                     ║ F
 G ║           + # +     ║ G ║                     ║ G
 H ║           # w #     ║ H ║                     ║ H
 I ║                     ║ I ║                     ║ I
 J ║                     ║ J ║           @         ║ J
   ╚[0-1-2-3-4-5-6-7-8-9]╝   ╚[0-1-2-3-4-5-6-7-8-9]╝

-- At the end of the board, Warriors can "Recall" back to their home rank.
-- This does not collide with any other pieces.
-- Internally, this is implemented as "Teleport Backwards exactly 9 tiles."

-------------------------------------------------------

                        Runner
                ╔[0-1-2-3-4-5-6-7-8-9]╗
              A ║ #               #   ║ A
              B ║   #           #     ║ B
              C ║     #       #       ║ C
              D ║       .   .         ║ D
              E ║         R           ║ E
              F ║       .   .         ║ F
              G ║     #       #       ║ G
              H ║   #           #     ║ H
              I ║ #               #   ║ I
              J ║                   # ║ J
                ╚[0-1-2-3-4-5-6-7-8-9]╝

-- Move only for 1st step, infinite Move/Capture beyond.

-------------------------------------------------------

                        Diplomat
                ╔[0-1-2-3-4-5-6-7-8-9]╗
              A ║                     ║ A
              B ║         .           ║ B
              C ║         .           ║ C
              D ║       % . %         ║ D
              E ║   . . . D . . .     ║ E
              F ║       % . %         ║ F
              G ║         .           ║ G
              H ║         .           ║ H
              I ║                     ║ I
              J ║                     ║ J
                ╚[0-1-2-3-4-5-6-7-8-9]╝

-- Converting a piece sacrifices the original Diplomat.

-------------------------------------------------------

        Blue Champion              Red Champion
   ╔[0-1-2-3-4-5-6-7-8-9]╗   ╔[0-1-2-3-4-5-6-7-8-9]╗
 A ║         .           ║ A ║           #         ║ A
 B ║         .           ║ B ║           #         ║ B
 C ║         .           ║ C ║           #         ║ C
 D ║         .           ║ D ║           #         ║ D
 E ║   # # # C # # #     ║ E ║         # # #       ║ E
 F ║       # # #         ║ F ║     # # # c # # #   ║ F
 G ║         #           ║ G ║           .         ║ G
 H ║         #           ║ H ║           .         ║ H
 I ║         #           ║ I ║           .         ║ I
 J ║         #           ║ J ║           .         ║ J
   ╚[0-1-2-3-4-5-6-7-8-9]╝   ╚[0-1-2-3-4-5-6-7-8-9]╝

 -- Infinite Move/Capture forward, Infinite Move back.

-------------------------------------------------------

                        General
                ╔[0-1-2-3-4-5-6-7-8-9]╗
              A ║ #       #       #   ║ A
              B ║   #     #     #     ║ B
              C ║     #   #   #       ║ C
              D ║       # # #         ║ D
              E ║ # # # # G # # # # # ║ E
              F ║       # # #         ║ F
              G ║     #   #   #       ║ G
              H ║   #     #     #     ║ H
              I ║ #       #       #   ║ I
              J ║         #         # ║ J
                ╚[0-1-2-3-4-5-6-7-8-9]╝

       -- Infinite Move/Capture in 8 directions.

-------------------------------------------------------

                        Stones
                ╔[0-1-2-3-4-5-6-7-8-9]╗
              A ║                     ║ A
              B ║                     ║ B
              C ║ S   S         S   S ║ C
              D ║                     ║ D
              E ║                     ║ E
              F ║                     ║ F
              G ║                     ║ G
              H ║ s   s         s   s ║ H
              I ║                     ║ I
              J ║                     ║ J
                ╚[0-1-2-3-4-5-6-7-8-9]╝

                -- Stones cannot move.
```

# Setup

### Components:

-   1 10x10 Board
-   2x Piece Set (_Red_ and _Blue_)
-   In each set:
    -   20x Blank
    -   10x Warrior
    -   5x Runner
    -   5x Diplomat
    -   4x Champion\*
    -   2x General\*
    -   4x Stone\*

> \* Because Diplomats can convert pieces, there can be up to 5 Champions, 3 Generals, or 8 Stones on one side at a time.
>
> This is solved the same way Physical Chess solves having up to 9 Queens: _by not solving it_

The starting arrangement is as follows:

```hs
   ╔[0-1-2-3-4-5-6-7-8-9]╗
 A ║ B B B B B B B B B B ║ A
 B ║ B B B B B B B B B B ║ B
 C ║ S   S         S   S ║ C
 D ║                     ║ D
 E ║         : :         ║ E
 F ║         : :         ║ F
 G ║                     ║ G
 H ║ s   s         s   s ║ H
 I ║ b b b b b b b b b b ║ I
 J ║ b b b b b b b b b b ║ J
   ╚[0-1-2-3-4-5-6-7-8-9]╝
```

# Merging

Notice how the board starts with **just Blanks and Stones.**

Blanks that are next to each other and outside of their starting 2 rows can **Merge** into another piece on any of the blanks' squares.

```hs
   ╔[0-1-2-3-4-5-6-7-8-9]╗
 A ║                     ║ A
 B ║  [B]                ║ B
 C ║   B    (B)          ║ C
 D ║   B       B B       ║ D
 E ║         B B         ║ E
 F ║                     ║ F
 G ║                     ║ G
 H ║           B B       ║ H
 I ║         B B         ║ I
 J ║           B         ║ J
   ╚[0-1-2-3-4-5-6-7-8-9]╝

-- [B1] cannot merge; it is within Blue's territory.
-- (C4) does not connect to D5.
-- [H5|H6|I4|I5]=>(J5) is valid; Blue can merge inside Red's territory.
```

Different pieces cost different numbers of Blanks. The costs are as follows:

|  Piece   | Cost |
| :------: | :--: |
| Warrior  |  2   |
|  Runner  |  4   |
| Diplomat |  4   |
| Champion |  5   |
| General  |  10  |
|  Stone   | 21\* |

> \* the fact that stones can be made with 21 blanks is a totally necessary feature and is definitely not just a gimmick
>
> dude trust me

# Turns

Players take turns moving, as in other abstract games. **Blue goes first.** Yes, the player on **top**, not the player on the bottom.

At the start of each turn, your _Energy_ is calculated as the number of **Stones** you have - at the start, it's 4:

```hs
Energy: # # # #

   ╔[0-1-2-3-4-5-6-7-8-9]╗
 A ║ B B B B B B B B B B ║ A
 B ║ B B B B B B B B B B ║ B
 C ║(S) (S)       (S) (S)║ C
 D ║                     ║ D
 E ║         : :         ║ E
 F ║         : :         ║ F
 G ║                     ║ G
 H ║ s   s         s   s ║ H
 I ║ b b b b b b b b b b ║ I
 J ║ b b b b b b b b b b ║ J
   ╚[0-1-2-3-4-5-6-7-8-9]╝
```

Once your energy runs out, your turn ends.

All normal piece moves cost 1 Energy. _You cannot move the same piece twice in the same turn._

Merging will cost you as much Energy as the **number of Blanks** used. If you do not have enough Energy, you may still merge, though this of course ends your turn. "Negative Energy" does **not** carry over to the next turn.

Newly **merged** or **converted** pieces _can move on the same turn,_ provided the player has any remaining energy.

# Endings

### You Win if:

1. Your opponent **Resigns.**
2. Your opponent has **no Stones** left.
3. Your opponent has **no** non-stone **Pieces** left.
4. You occupy **all 4 Victory Tiles** at once.

### The game is Drawn by:

1. A mutual **agreement.**
2. Making **64 full turns** without a **Blank** move, a **Merge,** or a **Capture.**
3. **Fourfold** repetition of the position.
4. Both players being unable to achieve the win conditions. **(Not Implemented, just manually offer a Draw.)**
5. Stalemate, though this is extremely rare and achieving it probably requires breaking **YLK rule 16.3.1;** _"Bringing the game into disrepute"._
