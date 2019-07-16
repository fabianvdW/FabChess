# FabChess
UCI compliant chess engine in Rust.
Features:
## Movegeneration
Magic bitboards for slider pieces and look-up tables for the rest of the pieces.
Using only legal move generation.
Speed about 100MNPS with bulk counting, about 20-25MNPS without bulk counting in perft 6 from startpos.
## Search
Principal Variaton Search
Aspiration Window
Null Move Pruning
Futility Pruning
Late move reductions
Check extensions
Move sorting with: Relative History Heuristic, Killer Heuristic, TT Lookup
Quiesence search with SEE Pruning and standing pat.

## Evaluation
Is yet to support Texel tuning which will probaply increase Elo by quite a margin.
Tapered eval from Stockfish.
Safety Table is also still a copy from Stockfish's implementation.

## Referee
FabChess also comes with a referee which supports 1 vs 1 Engine tourneys for any UCI compliant chess engine.
The referee can load any opening and supports any Timecontrol.
It can also load different epd testsuits (such as the Strategic Test suite).
Inspired by:

- https://www.chessprogramming.org/Main_Page
- Stockfish
- Ethereal
