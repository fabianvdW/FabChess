pub const STD_FEN:&str ="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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