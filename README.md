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
