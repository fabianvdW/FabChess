# FabChess
UCI compliant chess engine in Rust.

This is only an engine, connect it with a gui for real usage.

## Wiki
Interested in how it works?

Check out the wiki at: https://github.com/fabianvdW/FabChess/wiki
## Setup
Download latest realease for your OS in the release section.

If you want to compile from source, make sure you have the latest version of Rust and Cargo installed, then do
```
git clone https://github.com/fabianvdW/FabChess.git
cd FabChess
cargo run --release
...
uci
< id name FabChess v1.10
< id author Fabian von der Warth, Contributor: Erik Imgrund
< uciok
go infinite
...
```

## Usage
FabChess supports more commands than the standard UCI specifies.
### Debug print
Use `d` for a debug print of the board
```
> position startpos
> d
<
+---+---+---+---+---+---+---+---+
| r | n | b | q | k | b | n | r |
+---+---+---+---+---+---+---+---+
| p | p | p | p | p | p | p | p |
+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |
+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |
+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |
+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |
+---+---+---+---+---+---+---+---+
| P | P | P | P | P | P | P | P |
+---+---+---+---+---+---+---+---+
| R | N | B | Q | K | B | N | R |
+---+---+---+---+---+---+---+---+
Castle Rights:
White Kingside: true
White Queenside: true
Black Kingside: true
Black Queenside: true
En Passant Possible: 0
Half-Counter: 0
Full-Counter: 1
Side to Move: 0
Hash: 7954168898935982804

FEN: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
```
### Static evaluation
Use `static` to get a static evaluation for the position
```
> position startpos
> static
< cp 38
```
### Display evaluation
If you compile FabChess with an extra flag, it will also write a detailed overview of the evaluation to a logfile.

!!! Make sure not to run any go command with this, else it will quite literally produce a lot of text !!!
```
> cargo run --features "display-eval"
> static
< cp 38
```
Logfile called `log.txt`:
```
PSQT for White:
	Pawns  : (-220 , -162)
	Knights: (-98 , -89)
	Bishops: (-27 , -35)
	King   : (29 , -77)
Sum: (-316 , -363)

PSQT for Black:
	Pawns  : (-220 , -162)
	Knights: (-98 , -89)
	Bishops: (-27 , -35)
	King   : (21 , -76)
Sum: (-324 , -362)

PSQT for White:
	Pawns  : (-220 , -162)
	Knights: (-98 , -89)
	Bishops: (-27 , -35)
	King   : (29 , -77)
Sum: (-316 , -363)

PSQT for Black:
	Pawns  : (-220 , -162)
	Knights: (-98 , -89)
	Bishops: (-27 , -35)
	King   : (21 , -76)
Sum: (-324 , -362)

MG PSQT Sum: 8
EG PSQT Sum: -1

Knights for White:
	Supported by pawns: 0 -> (0 , 0)
	Outposts: 0 -> (0 , 0)
Sum: (0 , 0)

Knights for Black:
	Supported by pawns: 0 -> (0 , 0)
	Outposts: 0 -> (0 , 0)
Sum: (0 , 0)

MG Knight Sum: 0 - 0 -> 0
EG Knight Sum: 0 - 0 -> 0

Piecewise for White:
	Mobility Knight: (-12 , -6)
	Mobility Bishop: (-20 , -60)
	Bishop Diagonally Adj: (-68 , 0)
	Mobility Rook  : (-56 , -60)
	Mobility Queen : (-23 , -40)
	Rooks on open  : 0 -> (0 , 0)
	Rooks on seventh: 0 -> (0 , 0)
	Knight Attackers/Value: (0 , 0)
	Bishop Attackers/Value: (0 , 0)
	Rook Attackers/Value: (0 , 0)
	Queen Attackers/Value: (0 , 0)
	Sum Attackers/Value: (0 , 0)
	Attack value: 0 * 0 / 100.0 -> 0
Sum: (-179 , -166)

Piecewise for Black:
	Mobility Knight: (-12 , -6)
	Mobility Bishop: (-20 , -60)
	Bishop Diagonally Adj: (-68 , 0)
	Mobility Rook  : (-56 , -60)
	Mobility Queen : (-23 , -40)
	Rooks on open  : 0 -> (0 , 0)
	Rooks on seventh: 0 -> (0 , 0)
	Knight Attackers/Value: (0 , 0)
	Bishop Attackers/Value: (0 , 0)
	Rook Attackers/Value: (0 , 0)
	Queen Attackers/Value: (0 , 0)
	Sum Attackers/Value: (0 , 0)
	Attack value: 0 * 0 / 100.0 -> 0
Sum: (-179 , -166)

MG Piecewise Sum: -179 - -179 -> 0
EG Piecewise Sum: -166 - -166 -> 0

King for White:
	Shield pawn missing: 0 -> (10 , -3)
	Shield pawn on open file missing: 0 -> (-38 , -9)
Sum: (-28 , -12)

King for Black:
	Shield pawn missing: 0 -> (10 , -3)
	Shield pawn on open file missing: 0 -> (-38 , -9)
Sum: (-28 , -12)

MG King Sum: -28 - -28 -> 0
EG King Sum: -12 - -12 -> 0

Pawns for White:
	Doubled: 0 -> (0 , 0)
	Isolated: 0 -> (0 , 0)
	Backward: 0 -> (0 , 0)
	Supported: 0 -> (0 , 0)
	Attack Center: 0 -> (0 , 0)
	Mobility: 30 -> (180 , 270)
	Passer Blocked/Not Blocked: 0 , 0 -> MG/EG(0 , 0)
	Rook behind passer: 0 -> (0 , 0)
	Enemy Rook behind passer: 0 -> (0 , 0)
Sum: (180 , 270)

Pawns for Black:
	Doubled: 0 -> (0 , 0)
	Isolated: 0 -> (0 , 0)
	Backward: 0 -> (0 , 0)
	Supported: 0 -> (0 , 0)
	Attack Center: 0 -> (0 , 0)
	Mobility: 30 -> (180 , 270)
	Passer Blocked/Not Blocked: 0 , 0 -> MG/EG(0 , 0)
	Rook behind passer: 0 -> (0 , 0)
	Enemy Rook behind passer: 0 -> (0 , 0)
Sum: (180 , 270)

MG Pawn Sum: 180 - 180 -> 0
EG Pawn Sum: 270 - 270 -> 0

Piece values for White
	Pawns: 8 -> (816 , 1168)
	Knights: 2 -> (920 , 1184)
	Bishops: 2 -> (928 , 1112)
	Bishop-Pair: 1 -> (30 , 82)
	Rooks: 2 -> (1242 , 1876)
	Queens: 1 -> (1445 , 1814)
Sum: (5381 , 7236)

Piece values for Black
	Pawns: 8 -> (816 , 1168)
	Knights: 2 -> (920 , 1184)
	Bishops: 2 -> (928 , 1112)
	Bishop-Pair: 1 -> (30 , 82)
	Rooks: 2 -> (1242 , 1876)
	Queens: 1 -> (1445 , 1814)
Sum: (5381 , 7236)

MG Piece value Sum: 5381 - 5381 -> 0
EG Piece value Sum: 7236 - 7236 -> 0

Tempo:(30 , 32)

MG Sum: 8 + 0 + 0 + 0 + 0 + 0 + 30 -> 38

EG Sum: -1 + 0 + 0 + 0 + 0 + 0 + 32 -> 31
Phase: 128

Final Result: (38 * 128 + 31 * (128.0 - 128))/128.0 -> 38
```
### Perft
## Inspired by:

- https://www.chessprogramming.org/Main_Page
- Stockfish
- Ethereal
- Asymptote
