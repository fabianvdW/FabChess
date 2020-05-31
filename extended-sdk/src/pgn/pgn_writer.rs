extern crate chrono;
extern crate hostname;
extern crate rand;
use chrono::Local;
use core_sdk::board_representation::game_state::*;
use core_sdk::move_generation::makemove::make_move;

pub struct PGNMetadata {
    pub event_name: Option<String>,
    pub site: Option<String>,
    pub date: Option<String>,
    pub round: Option<String>,
    pub white: Option<String>,
    pub black: Option<String>,
    pub result: Option<String>,
    pub termination: Option<String>,
    pub starting_position: String,
}
impl PGNMetadata {
    pub fn fill_systemdata(&mut self) {
        let date = Local::now();
        self.date = Some(date.format("%Y.%m.%d").to_string());
        self.site = Some(hostname::get().unwrap().into_string().unwrap());
    }
}
impl Default for PGNMetadata {
    fn default() -> Self {
        PGNMetadata {
            event_name: None,
            site: None,
            date: None,
            round: None,
            white: None,
            black: None,
            result: None,
            termination: None,
            starting_position: crate::misc::STD_FEN.to_owned(),
        }
    }
}
pub fn get_pgn_string(
    metadata: &PGNMetadata,
    moves: Vec<GameMove>,
    opening_comment: Option<usize>,
) -> String {
    let mut res_str = String::new();
    let s = if metadata.event_name.is_some() {
        metadata.event_name.clone().unwrap()
    } else {
        String::from("?")
    };
    res_str.push_str(&format!("[Event \"{}\"]\n", s));
    let s = if metadata.site.is_some() {
        metadata.site.clone().unwrap()
    } else {
        String::from("?")
    };
    res_str.push_str(&format!("[Site \"{}\"]\n", s));
    let s = if metadata.date.is_some() {
        metadata.date.clone().unwrap()
    } else {
        String::from("????.??.??")
    };
    res_str.push_str(&format!("[Date \"{}\"]\n", s));
    let s = if metadata.round.is_some() {
        metadata.round.clone().unwrap()
    } else {
        String::from("?")
    };
    res_str.push_str(&format!("[Round \"{}\"]\n", s));
    let s = if metadata.white.is_some() {
        metadata.white.clone().unwrap()
    } else {
        String::from("?")
    };
    res_str.push_str(&format!("[White \"{}\"]\n", s));
    let s = if metadata.black.is_some() {
        metadata.black.clone().unwrap()
    } else {
        String::from("?")
    };
    res_str.push_str(&format!("[Black \"{}\"]\n", s));
    let s = if metadata.result.is_some() {
        metadata.result.clone().unwrap()
    } else {
        String::from("*")
    };
    res_str.push_str(&format!("[Result \"{}\"]\n", s));

    if let Some(s) = &metadata.termination {
        res_str.push_str(&format!("[Termination \"{}\"]\n", s));
    }
    if metadata.starting_position != crate::misc::STD_FEN {
        res_str.push_str("[SetUp \"1\"]\n");
        res_str.push_str(&format!("[FEN \"{}\"]\n", metadata.starting_position));
    }
    res_str.push_str(&format!("[PlyCount \"{}\"]\n", moves.len()));
    res_str.push_str("[WhiteType \"program\"]\n");
    res_str.push_str("[BlackType \"program\"]\n");
    res_str.push_str("\n");

    //Move Text section
    let mut start_pos = GameState::from_fen(&metadata.starting_position);
    let mut move_text = String::new();
    if start_pos.get_color_to_move() == BLACK {
        move_text.push_str(&format!("{}... ", start_pos.get_full_moves()));
    } else {
        move_text.push_str(&format!("{}. ", start_pos.get_full_moves()));
    }
    if opening_comment.is_some() && opening_comment.unwrap() == 0 {
        move_text.push_str("{Opening book has ended} ");
    }
    let mut current_color = start_pos.get_color_to_move();
    for (index, mv) in moves.iter().enumerate() {
        move_text.push_str(&format!("{} ", mv.to_san(&start_pos)));
        if opening_comment.is_some() && (index + 1) == opening_comment.unwrap() {
            move_text.push_str("{Opening book has ended} ");
        }
        start_pos = make_move(&start_pos, *mv);
        if current_color == BLACK && index < moves.len() - 1 {
            move_text.push_str(&format!("{}. ", start_pos.get_full_moves()));
        }
        current_color = 1 - current_color;
    }
    move_text.push_str(if metadata.result.is_some() {
        metadata.result.as_ref().unwrap()
    } else {
        "*"
    });
    let contents: Vec<&str> = move_text.split_whitespace().collect();
    //Make sure that every line is only 80 long at maximum
    let mut move_text = String::new();
    let mut current_line = String::new();
    for (index, content) in contents.iter().enumerate() {
        if current_line.chars().count() + content.chars().count() >= 80 {
            current_line = current_line[..current_line.len() - 1].to_owned();
            move_text.push_str(&format!("{}\n", current_line));
            current_line.clear();
        }
        if index == contents.len() - 1 {
            current_line.push_str(content);
        } else {
            current_line.push_str(&format!("{} ", content));
        }
    }
    move_text.push_str(&current_line);
    res_str.push_str(&move_text);
    res_str.push_str("\n\n");
    res_str
}
#[cfg(test)]
mod tests {
    use crate::pgn::pgn_writer::PGNMetadata;
    use core_sdk::board_representation::game_state::*;
    use core_sdk::move_generation::makemove::make_move;
    use core_sdk::move_generation::movegen;
    use rand::Rng;

    #[test]
    fn pgn_writer_test() {
        let mut movelist = movegen::MoveList::default();
        let mut rng = rand::thread_rng();
        let mut g = GameState::from_fen("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq -");
        let mut moves = Vec::with_capacity(100);
        let mut res = GameResult::Ingame;
        loop {
            let agsi = movegen::generate_moves(&g, false, &mut movelist);
            if movelist.move_list.is_empty() || g.get_half_moves() >= 100 {
                if g.get_half_moves() >= 100 {
                    res = GameResult::Draw;
                }
                if movelist.move_list.is_empty() {
                    if agsi.stm_incheck {
                        if g.get_color_to_move() == WHITE {
                            res = GameResult::BlackWin;
                        } else {
                            res = GameResult::WhiteWin;
                        }
                    } else {
                        res = GameResult::Draw;
                    }
                }
                break;
            }
            let mv = movelist.move_list[rng.gen_range(0, movelist.move_list.len())];
            g = make_move(&g, mv.0);
            moves.push(mv.0);
        }
        let mut metadata = PGNMetadata::default();
        metadata.fill_systemdata();
        metadata.result = Some(res.to_string());
        metadata.starting_position =
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq -".to_owned();
        print!("{}", super::get_pgn_string(&metadata, moves, Some(0)))
    }
}
