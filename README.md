# FabChess
UCI compliant chess engine in Rust.

This is only an engine, connect it with a gui for real usage.

## Wiki
Interested in how it works?

Check out the wiki at: https://github.com/fabianvdW/FabChess/wiki
## Setup
Download latest release for your OS in the release section.

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
For a faster compile including popcount operation for new processors, run
```
cargo rustc --release --bin fabchess -- -C target-cpu=native
```
The binary will be in `./target/release`
## Playing strength
| Version       | 40/4    |  40/40 | Comment |
|---------------|---------|--------|---------|
| Latest dev    |  -      |   -    |         |
| Version 1.12.6| 2785    |  2760  | Estimate|
| Version 1.12  | 2785    |  -     | See CCRL|
| Version 1.11  |  -      |  2606  | See CCRL|
| Version 1.10  | >2600   |        | Estimate|
| Version 1.9.1 | 2510    |  2442  | See CCRL|
| Version 1.9   | 2534    |  2463  | See CCRL|
| Version 1.8   | 2409    |   -    | See CCRL|
## Usage
FabChess supports more commands than the standard UCI specifies.

### Static evaluation
Use `static` to get a static evaluation of the position
```
> position startpos
> static
< cp 32
```
### Display evaluation
If you compile FabChess with an extra flag, it will also write a detailed overview of the evaluation to a logfile.

!!! Make sure not to run any `go` command with this, else it will quite literally produce a lot of text !!!
```
> cargo run --features "display-eval"
> position startpos
> static
< cp 32
```
Logfile called `log.txt`:
```
Evaluating GameState fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

PSQT for White:
	Pawns  : (-243 , -174)
	Knights: (-93 , -92)
	Bishops: (-31 , -38)
	King   : (32 , -97)
Sum: (-335 , -401)

PSQT for Black:
	Pawns  : (-243 , -174)
	Knights: (-93 , -92)
	Bishops: (-31 , -38)
	King   : (32 , -97)
Sum: (-335 , -401)

MG PSQT Sum: 0
EG PSQT Sum: 0

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
	Mobility Knight: (-14 , 0)
	Mobility Bishop: (-18 , -58)
	Bishop Diagonally Adj: (-52 , 48)
	Mobility Rook  : (-50 , -60)
	Mobility Queen : (-22 , -40)
	Rooks on open  : 0 -> (0 , 0)
	Rooks on seventh: 0 -> (0 , 0)
	Knight Attackers/Value: (0 , 0)
	Bishop Attackers/Value: (0 , 0)
	Rook Attackers/Value: (0 , 0)
	Queen Attackers/Value: (0 , 0)
	Sum Attackers/Value: (0 , 0)
	Attack MG value: 0 * 0 / 100.0 -> 0
	Attack EG value: 0 * 0 / 100.0 -> 0
Sum: (-156 , -110)

Piecewise for Black:
	Mobility Knight: (-14 , 0)
	Mobility Bishop: (-18 , -58)
	Bishop Diagonally Adj: (-52 , 48)
	Mobility Rook  : (-50 , -60)
	Mobility Queen : (-22 , -40)
	Rooks on open  : 0 -> (0 , 0)
	Rooks on seventh: 0 -> (0 , 0)
	Knight Attackers/Value: (0 , 0)
	Bishop Attackers/Value: (0 , 0)
	Rook Attackers/Value: (0 , 0)
	Queen Attackers/Value: (0 , 0)
	Sum Attackers/Value: (0 , 0)
	Attack MG value: 0 * 0 / 100.0 -> 0
	Attack EG value: 0 * 0 / 100.0 -> 0
Sum: (-156 , -110)

MG Piecewise Sum: -156 - -156 -> 0
EG Piecewise Sum: -110 - -110 -> 0

King for White:
	Shield pawn missing: 0 -> (6 , -8)
	Shield pawn on open file missing: 0 -> (-38 , -10)
Sum: (-32 , -18)

King for Black:
	Shield pawn missing: 0 -> (6 , -8)
	Shield pawn on open file missing: 0 -> (-38 , -10)
Sum: (-32 , -18)

MG King Sum: -32 - -32 -> 0
EG King Sum: -18 - -18 -> 0

Pawns for White:
	Doubled: 0 -> (0 , 0)
	Isolated: 0 -> (0 , 0)
	Backward: 0 -> (0 , 0)
	Supported: 0 -> (0 , 0)
	Attack Center: 0 -> (0 , 0)
	Mobility: 30 -> (210 , 360)
	Passer Blocked/Not Blocked: 0 , 0 -> MG/EG(0 , 0)
	Rook behind passer: 0 -> (0 , 0)
	Enemy Rook behind passer: 0 -> (0 , 0)
Sum: (210 , 360)

Pawns for Black:
	Doubled: 0 -> (0 , 0)
	Isolated: 0 -> (0 , 0)
	Backward: 0 -> (0 , 0)
	Supported: 0 -> (0 , 0)
	Attack Center: 0 -> (0 , 0)
	Mobility: 30 -> (210 , 360)
	Passer Blocked/Not Blocked: 0 , 0 -> MG/EG(0 , 0)
	Rook behind passer: 0 -> (0 , 0)
	Enemy Rook behind passer: 0 -> (0 , 0)
Sum: (210 , 360)

MG Pawn Sum: 210 - 210 -> 0
EG Pawn Sum: 360 - 360 -> 0

Piece values for White
	Pawns: 8 -> (848 , 1488)
	Knights: 2 -> (976 , 1416)
	Bishops: 2 -> (968 , 1300)
	Bishop-Pair: 1 -> (34 , 107)
	Rooks: 2 -> (1334 , 2286)
	Queens: 1 -> (1515 , 2137)
Sum: (5675 , 8734)

Piece values for Black
	Pawns: 8 -> (848 , 1488)
	Knights: 2 -> (976 , 1416)
	Bishops: 2 -> (968 , 1300)
	Bishop-Pair: 1 -> (34 , 107)
	Rooks: 2 -> (1334 , 2286)
	Queens: 1 -> (1515 , 2137)
Sum: (5675 , 8734)

MG Piece value Sum: 5675 - 5675 -> 0
EG Piece value Sum: 8734 - 8734 -> 0

Tempo:(32 , 42)

MG Sum: 0 + 0 + 0 + 0 + 0 + 0 + 32 -> 32

EG Sum: (0 + 0 + 0 + 0 + 0 + 0 + 42) /1.5 -> 28
Phase: 128

Final Result: (32 * 128 + 28 * (128.0 - 128))/128.0 -> 32
```
### Perft
You can run perft on an arbitrary position. Note that if there is no king on the board for either side or the position is otherwise illegal, FabChess will crash (intended).
```
> position startpos
> perft 6
< 
a2a3: 4463267
b2b3: 5310358
c2c3: 5417640
d2d3: 8073082
e2e3: 9726018
f2f3: 4404141
g2g3: 5346260
h2h3: 4463070
a2a4: 5363555
b2b4: 5293555
c2c4: 5866666
d2d4: 8879566
e2e4: 9771632
f2f4: 4890429
g2g4: 5239875
h2h4: 5385554
g1f3: 5723523
g1h3: 4877234
b1a3: 4856835
b1c3: 5708064
119060324
Time 1.222 (97430707.03764321 nps)
```
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
## Inspired by:

- https://www.chessprogramming.org/Main_Page
- Stockfish
- Ethereal
- Asymptote

Special thanks to Andrew Grant for writing [OpenBench](https://github.com/AndyGrant/OpenBench), which is used for selfplay testing lately.
