pub const STD_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const KING_BASE_PATH: [&str; 15] = [
    "./KingBase/KingBase2019-A00-A39.pgn",
    "./KingBase/KingBase2019-A40-A79.pgn",
    "./KingBase/KingBase2019-A80-A99.pgn",
    "./KingBase/KingBase2019-B00-B19.pgn",
    "./KingBase/KingBase2019-B20-B49.pgn",
    "./KingBase/KingBase2019-B50-B99.pgn",
    "./KingBase/KingBase2019-C00-C19.pgn",
    "./KingBase/KingBase2019-C20-C59.pgn",
    "./KingBase/KingBase2019-C60-C99.pgn",
    "./KingBase/KingBase2019-D00-D29.pgn",
    "./KingBase/KingBase2019-D30-D69.pgn",
    "./KingBase/KingBase2019-D70-D99.pgn",
    "./KingBase/KingBase2019-E00-E19.pgn",
    "./KingBase/KingBase2019-E20-E59.pgn",
    "./KingBase/KingBase2019-E60-E99.pgn",
];

#[allow(dead_code)]
pub fn to_string_board(board: u64) -> String {
    let mut res_str: String = String::new();
    res_str.push_str("+---+---+---+---+---+---+---+---+\n");
    for rank in 0..8 {
        res_str.push_str("| ");
        for file in 0..8 {
            let idx = 8 * (7 - rank) + file;
            if ((board >> idx) & 1) != 0 {
                res_str.push_str("X");
            } else {
                res_str.push_str(" ");
            }
            res_str.push_str(" | ");
        }
        res_str.push_str("\n+---+---+---+---+---+---+---+---+\n");
    }
    res_str
}
