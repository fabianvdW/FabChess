use crate::board_representation::game_state::{GameState, GameMove, GameMoveType, PieceType};

//2^20 Entrys
pub const CACHE_MASK: usize = 0x3FFFFF;
pub const CACHE_ENTRYS: usize = 4*1048576;

pub struct Cache {
    pub cache: Vec<Option<CacheEntry>>
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            cache: vec![None; CACHE_ENTRYS],
        }
    }
}

#[derive(Copy, Clone)]
pub struct CacheEntry {
    pub hash: u64,
    //64-Bit
    pub depth: i8,
    //8-Bit
    pub occurences: u8,
    //8-Bit
    pub plies_played: u16,
    //16-Bit
    pub score: f64,
    //64-Bit
    pub alpha: bool,
    //1-Bit
    pub beta: bool,
    //1-Bit
    pub mv: u16,//16-Bit
    //Insg 178-Bit 23-Bytes
}

impl CacheEntry {
    pub fn new(game_state: &GameState, depth_left: isize, score: f64, alpha: bool, beta: bool, mv: &GameMove) -> CacheEntry {
        CacheEntry {
            hash: game_state.hash,
            depth: depth_left as i8,
            occurences: 0,
            plies_played: ((game_state.full_moves - 1) * 2 + game_state.color_to_move) as u16,
            score,
            alpha,
            beta,
            mv: CacheEntry::mv_to_u16(&mv),
        }
    }

    pub fn mv_to_u16(mv: &GameMove) -> u16 {
        let mut res = 0usize;
        res |= mv.from << 10;
        res |= mv.to << 4;
        res |= match &mv.move_type {
            GameMoveType::Quiet => 1,
            GameMoveType::Castle => 2,
            GameMoveType::Promotion(a, _) => {
                match a {
                    PieceType::Queen => 3,
                    PieceType::Rook => 4,
                    PieceType::Bishop => 5,
                    PieceType::Knight => 6,
                    _ => panic!("Invalid promotion!")
                }
            }
            GameMoveType::Capture(_) => 7,
            GameMoveType::EnPassant => 8,
        };
        res as u16
    }

    pub fn u16_to_mv(mv: u16, game_state: &GameState) -> GameMove {
        let typ = mv & 15;
        let from = ((mv & 0xFC00) >> 10) as usize;
        let from_board = 1u64 << from;
        let to = ((mv & 0x03F0) >> 4) as usize;
        let to_board = 1u64 << to;
        let color_to_move = game_state.color_to_move;
        let enemy_color = 1 - color_to_move;
        let piece_type = if (game_state.pieces[0][color_to_move] & from_board) != 0u64 {
            PieceType::Pawn
        } else if (game_state.pieces[1][color_to_move] & from_board) != 0u64 {
            PieceType::Knight
        } else if (game_state.pieces[2][color_to_move] & from_board) != 0u64 {
            PieceType::Bishop
        } else if (game_state.pieces[3][color_to_move] & from_board) != 0u64 {
            PieceType::Rook
        } else if (game_state.pieces[4][color_to_move] & from_board) != 0u64 {
            PieceType::Queen
        } else {
            PieceType::King
        };
        if typ == 1 {
            return GameMove {
                from,
                to,
                piece_type,
                move_type: GameMoveType::Quiet,
            };
        } else if typ == 2 {
            debug_assert_eq!(piece_type, PieceType::King);
            return GameMove {
                from,
                to,
                piece_type,
                move_type: GameMoveType::Castle,
            };
        } else {
            if typ == 8 {
                return GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::EnPassant,
                };
            }
            let captured_piece_type = if (game_state.pieces[0][enemy_color] & to_board) != 0u64 {
                PieceType::Pawn
            } else if (game_state.pieces[1][enemy_color] & to_board) != 0u64 {
                PieceType::Knight
            } else if (game_state.pieces[2][enemy_color] & to_board) != 0u64 {
                PieceType::Bishop
            } else if (game_state.pieces[3][enemy_color] & to_board) != 0u64 {
                PieceType::Rook
            } else if (game_state.pieces[4][enemy_color] & to_board) != 0u64 {
                PieceType::Queen
            } else {
                PieceType::King
            };
            if typ == 3 {
                return GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::Promotion(PieceType::Queen, if captured_piece_type != PieceType::King { Some(captured_piece_type) } else { None }),
                };
            } else if typ == 4 {
                return GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::Promotion(PieceType::Rook, if captured_piece_type != PieceType::King { Some(captured_piece_type) } else { None }),
                };
            } else if typ == 5 {
                return GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::Promotion(PieceType::Bishop, if captured_piece_type != PieceType::King { Some(captured_piece_type) } else { None }),
                };
            } else if typ == 6 {
                return GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::Promotion(PieceType::Knight, if captured_piece_type != PieceType::King { Some(captured_piece_type) } else { None }),
                };
            } else {
                return GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::Capture(captured_piece_type),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CacheEntry;
    use crate::board_representation::game_state::{GameState, GameMove, GameMoveType, PieceType};
    use crate::move_generation::movegen;

    #[test]
    fn mv_to_u16_test() {
        let mut game_state = GameState::from_fen("k4b2/2p1P3/8/3P4/6b1/7P/8/R3K2R w KQ -");
        {
            let h3h4 = GameMove {
                from: 23,
                to: 31,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Quiet,
            };
            let h3h4u16 = CacheEntry::mv_to_u16(&h3h4);
            let h3h4res = CacheEntry::u16_to_mv(h3h4u16, &game_state);
            assert_eq!(h3h4res.move_type, h3h4.move_type);
            assert_eq!(h3h4res.piece_type, h3h4.piece_type);
            assert_eq!(h3h4res.from, h3h4.from);
            assert_eq!(h3h4res.to, h3h4.to);
        }
        {
            let h3g4 = GameMove {
                from: 23,
                to: 30,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Capture(PieceType::Bishop),
            };
            let h3g4u16 = CacheEntry::mv_to_u16(&h3g4);
            let h3g4res = CacheEntry::u16_to_mv(h3g4u16, &game_state);
            assert_eq!(h3g4res.from, h3g4.from);
            assert_eq!(h3g4res.to, h3g4.to);
            assert_eq!(h3g4res.move_type, h3g4.move_type);
            assert_eq!(h3g4res.piece_type, h3g4.piece_type);
        }
        {
            let e1c1 = GameMove {
                from: 4,
                to: 2,
                piece_type: PieceType::King,
                move_type: GameMoveType::Castle,
            };
            let e1c1u16 = CacheEntry::mv_to_u16(&e1c1);
            let e1c1res = CacheEntry::u16_to_mv(e1c1u16, &game_state);
            assert_eq!(e1c1res.from, e1c1.from);
            assert_eq!(e1c1res.to, e1c1.to);
            assert_eq!(e1c1res.move_type, e1c1.move_type);
            assert_eq!(e1c1res.piece_type, e1c1.piece_type);
        }
        {
            let e1g1 = GameMove {
                from: 4,
                to: 6,
                piece_type: PieceType::King,
                move_type: GameMoveType::Castle,
            };
            let e1g1u16 = CacheEntry::mv_to_u16(&e1g1);
            let e1g1res = CacheEntry::u16_to_mv(e1g1u16, &game_state);
            assert_eq!(e1g1res.from, e1g1.from);
            assert_eq!(e1g1res.to, e1g1.to);
            assert_eq!(e1g1res.move_type, e1g1.move_type);
            assert_eq!(e1g1res.piece_type, e1g1.piece_type);
        }
        {
            let e7e8q = GameMove {
                from: 52,
                to: 60,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Promotion(PieceType::Queen, None),
            };
            let e7e8qu16 = CacheEntry::mv_to_u16(&e7e8q);
            let e7e8qres = CacheEntry::u16_to_mv(e7e8qu16, &game_state);
            assert_eq!(e7e8qres.from, e7e8q.from);
            assert_eq!(e7e8qres.to, e7e8q.to);
            assert_eq!(e7e8qres.move_type, e7e8q.move_type);
            assert_eq!(e7e8qres.piece_type, e7e8q.piece_type);
        }
        {
            let e7e8r = GameMove {
                from: 52,
                to: 60,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Promotion(PieceType::Rook, None),
            };
            let e7e8ru16 = CacheEntry::mv_to_u16(&e7e8r);
            let e7e8rres = CacheEntry::u16_to_mv(e7e8ru16, &game_state);
            assert_eq!(e7e8rres.from, e7e8r.from);
            assert_eq!(e7e8rres.to, e7e8r.to);
            assert_eq!(e7e8rres.move_type, e7e8r.move_type);
            assert_eq!(e7e8rres.piece_type, e7e8r.piece_type);
        }
        {
            let e7e8b = GameMove {
                from: 52,
                to: 60,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Promotion(PieceType::Bishop, None),
            };
            let e7e8bu16 = CacheEntry::mv_to_u16(&e7e8b);
            let e7e8bres = CacheEntry::u16_to_mv(e7e8bu16, &game_state);
            assert_eq!(e7e8bres.from, e7e8b.from);
            assert_eq!(e7e8bres.to, e7e8b.to);
            assert_eq!(e7e8bres.move_type, e7e8b.move_type);
            assert_eq!(e7e8bres.piece_type, e7e8b.piece_type);
        }
        {
            let e7e8n = GameMove {
                from: 52,
                to: 60,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Promotion(PieceType::Knight, None),
            };
            let e7e8nu16 = CacheEntry::mv_to_u16(&e7e8n);
            let e7e8nres = CacheEntry::u16_to_mv(e7e8nu16, &game_state);
            assert_eq!(e7e8nres.from, e7e8n.from);
            assert_eq!(e7e8nres.to, e7e8n.to);
            assert_eq!(e7e8nres.move_type, e7e8n.move_type);
            assert_eq!(e7e8nres.piece_type, e7e8n.piece_type);
        }

        {
            let e7e8q = GameMove {
                from: 52,
                to: 61,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Promotion(PieceType::Queen, Some(PieceType::Bishop)),
            };
            let e7e8qu16 = CacheEntry::mv_to_u16(&e7e8q);
            let e7e8qres = CacheEntry::u16_to_mv(e7e8qu16, &game_state);
            assert_eq!(e7e8qres.from, e7e8q.from);
            assert_eq!(e7e8qres.to, e7e8q.to);
            assert_eq!(e7e8qres.move_type, e7e8q.move_type);
            assert_eq!(e7e8qres.piece_type, e7e8q.piece_type);
        }
        {
            let e7e8r = GameMove {
                from: 52,
                to: 61,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Promotion(PieceType::Rook, Some(PieceType::Bishop)),
            };
            let e7e8ru16 = CacheEntry::mv_to_u16(&e7e8r);
            let e7e8rres = CacheEntry::u16_to_mv(e7e8ru16, &game_state);
            assert_eq!(e7e8rres.from, e7e8r.from);
            assert_eq!(e7e8rres.to, e7e8r.to);
            assert_eq!(e7e8rres.move_type, e7e8r.move_type);
            assert_eq!(e7e8rres.piece_type, e7e8r.piece_type);
        }
        {
            let e7e8b = GameMove {
                from: 52,
                to: 61,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Promotion(PieceType::Bishop, Some(PieceType::Bishop)),
            };
            let e7e8bu16 = CacheEntry::mv_to_u16(&e7e8b);
            let e7e8bres = CacheEntry::u16_to_mv(e7e8bu16, &game_state);
            assert_eq!(e7e8bres.from, e7e8b.from);
            assert_eq!(e7e8bres.to, e7e8b.to);
            assert_eq!(e7e8bres.move_type, e7e8b.move_type);
            assert_eq!(e7e8bres.piece_type, e7e8b.piece_type);
        }
        {
            let e7e8n = GameMove {
                from: 52,
                to: 61,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Promotion(PieceType::Knight, Some(PieceType::Bishop)),
            };
            let e7e8nu16 = CacheEntry::mv_to_u16(&e7e8n);
            let e7e8nres = CacheEntry::u16_to_mv(e7e8nu16, &game_state);
            assert_eq!(e7e8nres.from, e7e8n.from);
            assert_eq!(e7e8nres.to, e7e8n.to);
            assert_eq!(e7e8nres.move_type, e7e8n.move_type);
            assert_eq!(e7e8nres.piece_type, e7e8n.piece_type);
        }
        game_state = movegen::make_move(&game_state, &GameMove {
            from: 23,
            to: 31,
            piece_type: PieceType::Pawn,
            move_type: GameMoveType::Quiet,
        });
        game_state = movegen::make_move(&game_state, &GameMove {
            from: 50,
            to: 34,
            piece_type: PieceType::Pawn,
            move_type: GameMoveType::Quiet,
        });
        {
            let d5d6 = GameMove {
                from: 35,
                to: 42,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::EnPassant,
            };
            let d5d6u16 = CacheEntry::mv_to_u16(&d5d6);
            let d5d6res = CacheEntry::u16_to_mv(d5d6u16, &game_state);
            assert_eq!(d5d6res.from, d5d6.from);
            assert_eq!(d5d6res.to, d5d6.to);
            assert_eq!(d5d6res.move_type, d5d6.move_type);
            assert_eq!(d5d6res.piece_type, d5d6.piece_type);
        }
    }
}