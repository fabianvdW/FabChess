# FabChess
UCI compliant chess engine in Rust.

This is only an engine, connect it with a gui for real usage.

## Wiki
Interested in how it works?

Check out the wiki at: https://github.com/fabianvdW/FabChess/wiki
## Setup
Download latest release for your OS in the release section.
## Compilation Guide
If you want to compile from source, make sure you have the latest version of Rust and Cargo installed. The only valid way to compile  the engine due to Rust's feature system having issues with workspaces is:
```
git clone https://github.com/fabianvdW/FabChess.git
cd FabChess
set RUSTFLAGS="-C target-cpu=native"
cargo run --release -p uci-engine
...
< info string Initialization Time: 0ms
> uci
< id name FabChess v1.15 BMI2
< id author Fabian von der Warth
< id contributors Erik Imgrund, Marcin Mielniczuk, Terje Kirstihagen
< option name Hash type spin default 256 min 0 max 131072
...
< uciok
go infinite
...
```
The binary can in `./target/release/uci-engine(.exe)`
## Playing strength
| Version       | 40/4    |  40/40 | Comment |
|---------------|---------|--------|---------|
| Current Dev   |         |        |         |
| Version 1.15  |3088/2982|  2943  | See CCRL|
| Version 1.14  |3017/2924|  2917  | See CCRL|
| Version 1.13  |2955-4CPU|  2877  | See CCRL|
| Version 1.12.6| 2788    |  2762  | See CCRL|
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
< cp 10
```
### Display evaluation
If you compile FabChess with an extra flag, it will also write a detailed overview of the evaluation to a logfile.
For this, you will have to change `core-sdk/Cargo.toml` to  include the feature in the default features:
`default = ["display-eval"]`


!!! Make sure not to run any `go` command with this, else it will quite literally produce a lot of text !!!
```
> cargo run -p uci-engine
> position startpos
> static
< cp 10
```
Stdout:
```
Evaluating GameState fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

Tempo:(10 , 13)

PSQT for White:
	Pawn  : (-228 , -144)
	Knight  : (-64 , -63)
	Bishop  : (-18 , -24)
	Rook  : (22 , -42)
	Queen  : (17 , -4)
	King  : (47 , -60)
Sum: (-224 , -337)

PSQT for Black:
	Pawn  : (-228 , -144)
	Knight  : (-64 , -63)
	Bishop  : (-18 , -24)
	Rook  : (22 , -42)
	Queen  : (17 , -4)
	King  : (47 , -60)
Sum: (-224 , -337)

PSQT Sum: (0 , 0)

Piece values for White
	Pawns: 8 -> (848 , 952)
	Knights: 2 -> (1016 , 1100)
	Bishops: 2 -> (984 , 948)
	Bishop-Pair: 1 -> (34 , 73)
	Rooks: 2 -> (1302 , 1730)
	Queens: 1 -> (1540 , 1631)
Sum: (5724 , 6434)

Piece values for Black
	Pawns: 8 -> (848 , 952)
	Knights: 2 -> (1016 , 1100)
	Bishops: 2 -> (984 , 948)
	Bishop-Pair: 1 -> (34 , 73)
	Rooks: 2 -> (1302 , 1730)
	Queens: 1 -> (1540 , 1631)
Sum: (5724 , 6434)

Piece value Sum: (5724 , 6434) - (5724 , 6434) -> (0 , 0)

Pawns for White:
	Doubled: 0 -> (0 , 0)
	Isolated: 0 -> (0 , 0)
	Backward: 0 -> (0 , 0)
	Supported: 0 -> (0 , 0)
	Attack Center: 0 -> (0 , 0)
	Mobility: 30 -> (180 , 300)
	Passer Blocked/Not Blocked: 0 , 0 -> (0 , 0)
	Rook behind passer: 0 -> (0 , 0)
	Enemy Rook behind passer: 0 -> (0 , 0)
	Weak passer: 0 -> (0 , 0)
	Passers distance to kings -> (0 , 0)
Sum: (180 , 300)

Pawns for Black:
	Doubled: 0 -> (0 , 0)
	Isolated: 0 -> (0 , 0)
	Backward: 0 -> (0 , 0)
	Supported: 0 -> (0 , 0)
	Attack Center: 0 -> (0 , 0)
	Mobility: 30 -> (180 , 300)
	Passer Blocked/Not Blocked: 0 , 0 -> (0 , 0)
	Rook behind passer: 0 -> (0 , 0)
	Enemy Rook behind passer: 0 -> (0 , 0)
	Weak passer: 0 -> (0 , 0)
	Passers distance to kings -> (0 , 0)
Sum: (180 , 300)

Pawn Sum: (180 , 300) - (180 , 300) -> (0 , 0)

Knights for White:
	Supported by pawns: 0 -> (0 , 0)
	Outposts: 0 -> (0 , 0)
Sum: (0 , 0)

Knights for Black:
	Supported by pawns: 0 -> (0 , 0)
	Outposts: 0 -> (0 , 0)
Sum: (0 , 0)

Knights Sum: (0 , 0) - (0 , 0) -> (0 , 0)

Piecewise for White:
	Mobility Knight: (-4 , 14)
	Mobility Bishop: (6 , -42)
	Bishop Diagonally Adj: (-48 , 70)
	Mobility Rook  : (-42 , -34)
	Mobility Queen : (-12 , -27)
	Rooks on open  : 0 -> (0 , 0)
	Rooks on semi-open  : 0 -> (0 , 0)
	Queens on open  : 0 -> (0 , 0)
	Queens on semi-open  : 0 -> (0 , 0)
	Rooks on seventh: 0 -> (0 , 0)
	Knight Attackers: Num: 0 , Val: (0 , 0)
	Bishop Attackers: Num: 0 , Val: (0 , 0)
	Rook Attackers: Num: 0 , Val: (0 , 0)
	Queen Attackers: Num: 0 , Val: (0 , 0)
	Sum Attackers: (Num: 0 , Val: (0 , 0)
	Attack MG value: 0 * 0 / 100.0 -> 0
	Attack EG value: 0 * -1 / 100.0 -> 0
Sum: (-100 , -19)

Piecewise for Black:
	Mobility Knight: (-4 , 14)
	Mobility Bishop: (6 , -42)
	Bishop Diagonally Adj: (-48 , 70)
	Mobility Rook  : (-42 , -34)
	Mobility Queen : (-12 , -27)
	Rooks on open  : 0 -> (0 , 0)
	Rooks on semi-open  : 0 -> (0 , 0)
	Queens on open  : 0 -> (0 , 0)
	Queens on semi-open  : 0 -> (0 , 0)
	Rooks on seventh: 0 -> (0 , 0)
	Knight Attackers: Num: 0 , Val: (0 , 0)
	Bishop Attackers: Num: 0 , Val: (0 , 0)
	Rook Attackers: Num: 0 , Val: (0 , 0)
	Queen Attackers: Num: 0 , Val: (0 , 0)
	Sum Attackers: (Num: 0 , Val: (0 , 0)
	Attack MG value: 0 * 0 / 100.0 -> 0
	Attack EG value: 0 * -1 / 100.0 -> 0
Sum: (-100 , -19)

Piecewise Sum: (-100 , -19) - (-100 , -19) -> (0 , 0)


King for White:
	Shield pawn missing: 0 -> (0 , -7)
	Shield pawn on open file missing: 0 -> (1 , 5)
Sum: (1 , -2)

King for Black:
	Shield pawn missing: 0 -> (0 , -7)
	Shield pawn on open file missing: 0 -> (1 , 5)
Sum: (1 , -2)

King Sum: (1 , -2) - (1 , -2) -> (0 , 0)

Sum: (0 , 0) + (0 , 0) + (0 , 0) + (0 , 0) + (0 , 0) + (0 , 0) + (10 , 13) -> (10 , 13) 
Phase: 128

Final Result: (10 * 128 + 13 * (128.0 - 128))/128.0 -> 10
cp 10
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
b1a3: 4856835
b1c3: 5708064
g1f3: 5723523
g1h3: 4877234
119060324
Time 0.901 (132142423.97336292 nps)
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
CWK: true
CWQ: true
CBK: true
CBQ: true
En Passant Possible: 0
Half-Counter: 0
Full-Counter: 1
Side to Move: 0
Hash: 5939436254971627240
FEN: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
```
## Inspired heavily by:

- https://www.chessprogramming.org/Main_Page
- Stockfish
- Ethereal
- Asymptote

Special thanks to Andrew Grant for writing [OpenBench](https://github.com/AndyGrant/OpenBench), which is used for selfplay testing, in order to incrementally improve on the last versions.
