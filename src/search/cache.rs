use crate::board_representation::game_state::{
    GameMove, GameMoveType, GameState, PieceType, BISHOP, KNIGHT, PAWN, QUEEN, ROOK,
};
use crate::search::{CombinedSearchParameters, SearchInstruction};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

pub struct Cache {
    pub entries: usize,
    pub locks: usize,
    pub entries_per_lock: usize,
    pub full: AtomicUsize,
    pub cache: Vec<RwLock<Vec<Option<CacheEntry>>>>,
}
pub const DEFAULT_LOCKS: usize = 1024;
pub const MIN_LOCKS: usize = 1;
pub const MAX_LOCKS: usize = 65536; // This is really the maximum!!!
                                    // Else we would need to index by upper_index = (hash >> 47 or lower)
                                    // Using a higher number will lead to the cache not being able to be used fully

pub const DEFAULT_HASH_SIZE: usize = 256; //IN MB
pub const MIN_HASH_SIZE: usize = 0; //IN MB
pub const MAX_HASH_SIZE: usize = 131072; //IN MB
impl Default for Cache {
    fn default() -> Self {
        let mut entries = DEFAULT_HASH_SIZE * 1024 * 1024 / 24;
        let entries_per_lock = entries / DEFAULT_LOCKS;
        entries = entries * DEFAULT_LOCKS;
        let mut cache = Vec::with_capacity(DEFAULT_LOCKS);
        for _ in 0..DEFAULT_LOCKS {
            cache.push(RwLock::new(vec![None; entries_per_lock]));
        }
        Cache {
            entries,
            locks: DEFAULT_LOCKS,
            entries_per_lock,
            full: AtomicUsize::new(0),
            cache,
        }
    }
}

impl Cache {
    pub fn with_size(mb_size: usize, locks: usize) -> Self {
        let mut entries = 1024 * 1024 * mb_size / 24;
        let entries_per_lock = entries / locks;
        entries = entries_per_lock * locks;
        let mut cache = Vec::with_capacity(locks);
        for _ in 0..locks {
            cache.push(RwLock::new(vec![None; entries_per_lock]));
        }
        Cache {
            entries,
            locks,
            entries_per_lock,
            full: AtomicUsize::new(0),
            cache,
        }
    }
    pub fn get_status(&self) -> f64 {
        self.full.load(Ordering::Relaxed) as f64 / self.entries as f64 * 1000.
    }
    pub fn clear(&self) {
        for bucket in &self.cache {
            let mut lock = bucket.write().unwrap();
            *lock = vec![None; self.entries_per_lock];
        }
        self.full.store(0, Ordering::Relaxed);
    }

    pub fn age_entry(&self, hash: u64, new_age: u16) {
        let upper_index = (hash >> 48) as usize % self.locks;
        let lock = unsafe { self.cache.get_unchecked(upper_index) };
        unsafe {
            lock.write()
                .unwrap()
                .get_unchecked_mut(hash as usize % self.entries_per_lock)
                .as_mut()
                .unwrap()
                .plies_played = new_age;
        }
    }
    pub fn get(&self, hash: u64) -> Option<CacheEntry> {
        let upper_index = (hash >> 48) as usize % self.locks;
        let lock = unsafe { self.cache.get_unchecked(upper_index) };
        unsafe {
            lock.read()
                .unwrap()
                .get_unchecked(hash as usize % self.entries_per_lock)
                .clone()
        }
    }

    pub fn insert(
        &self,
        p: &CombinedSearchParameters,
        mv: &GameMove,
        score: i16,
        original_alpha: i16,
        root_plies_played: usize,
        static_evaluation: Option<i16>,
    ) {
        if self.entries == 0 {
            return;
        }
        let lower_bound = score >= p.beta;
        let upper_bound = score <= original_alpha;
        let pv_node = p.beta - p.alpha > 1;
        let upper_index = (p.game_state.hash >> 48) as usize % self.locks;
        let index = p.game_state.hash as usize % self.entries_per_lock;
        //Aquire lock
        let lock = unsafe { self.cache.get_unchecked(upper_index) };
        let mut write = lock.write().unwrap();
        let ce = unsafe { write.get_unchecked(index) };
        if ce.is_none() {
            let new_entry = CacheEntry::new(
                p.game_state,
                p.depth_left,
                score,
                upper_bound,
                lower_bound,
                mv,
                static_evaluation,
                pv_node,
                root_plies_played as u16,
            );
            self.full
                .store(self.full.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
            write[index] = Some(new_entry);
        } else {
            let new_entry_val = f64::from(p.depth_left) * if !pv_node { 0.7 } else { 1.0 };
            let old_entry = ce.as_ref().unwrap();

            let old_entry_val = if old_entry.plies_played < root_plies_played as u16 {
                -1.0
            } else {
                f64::from(old_entry.depth) * if !old_entry.pv_node { 0.7 } else { 1.0 }
            };
            let state_plies_played = (p.game_state.full_moves - 1) * 2 + p.game_state.color_to_move;
            if state_plies_played == root_plies_played || old_entry_val <= new_entry_val {
                let new_entry = CacheEntry::new(
                    p.game_state,
                    p.depth_left,
                    score,
                    upper_bound,
                    lower_bound,
                    mv,
                    static_evaluation,
                    pv_node,
                    root_plies_played as u16,
                );
                write[index] = Some(new_entry);
            }
        }
    }

    pub fn lookup(
        &self,
        p: &CombinedSearchParameters,
        static_evaluation: &mut Option<i16>,
        tt_move: &mut Option<GameMove>,
        root_plies: usize,
    ) -> SearchInstruction {
        if self.entries == 0 {
            return SearchInstruction::ContinueSearching;
        }
        let ce = self.get(p.game_state.hash);
        if let Some(ce) = ce {
            if ce.hash == p.game_state.hash {
                if ce.depth >= p.depth_left as i8
                    && (p.beta - p.alpha <= 1 || p.depth_left <= 0)
                    && (!ce.alpha && !ce.beta
                        || ce.beta && ce.score >= p.beta
                        || ce.alpha && ce.score <= p.alpha)
                {
                    *tt_move = Some(CacheEntry::u16_to_mv(ce.mv, p.game_state));
                    return SearchInstruction::StopSearching(ce.score);
                }
                *static_evaluation = ce.static_evaluation;
                let mv = CacheEntry::u16_to_mv(ce.mv, p.game_state);
                *tt_move = Some(mv);
                if ce.plies_played != root_plies as u16 {
                    self.age_entry(p.game_state.hash, root_plies as u16);
                }
            }
        }
        SearchInstruction::ContinueSearching
    }
}

#[derive(Copy, Clone)]
pub struct CacheEntry {
    pub hash: u64,
    //64 bits
    pub depth: i8,
    //8 bits
    pub plies_played: u16,
    //16 bits
    pub score: i16,
    //16 bits
    pub alpha: bool,
    //8 bits
    pub beta: bool,
    //8 bits
    pub mv: u16,
    //16 bits
    pub static_evaluation: Option<i16>,
    //16 bits
    pub pv_node: bool, //Summed 160 bits 20 bytes
}

impl CacheEntry {
    pub fn new(
        game_state: &GameState,
        depth_left: i16,
        score: i16,
        alpha: bool,
        beta: bool,
        mv: &GameMove,
        static_evaluation: Option<i16>,
        pv_node: bool,
        root_plies: u16,
    ) -> CacheEntry {
        CacheEntry {
            hash: game_state.hash,
            depth: depth_left as i8,
            plies_played: root_plies,
            score,
            alpha,
            beta,
            mv: CacheEntry::mv_to_u16(&mv),
            static_evaluation,
            pv_node,
        }
    }

    #[inline(always)]
    pub fn mv_to_u16(mv: &GameMove) -> u16 {
        let mut res = 0;
        res |= mv.from << 10;
        res |= mv.to << 4;
        res |= match &mv.move_type {
            GameMoveType::Quiet => 1,
            GameMoveType::Castle => 2,
            GameMoveType::Promotion(a, _) => match a {
                PieceType::Queen => 3,
                PieceType::Rook => 4,
                PieceType::Bishop => 5,
                PieceType::Knight => 6,
                _ => panic!("Invalid promotion!"),
            },
            GameMoveType::Capture(_) => 7,
            GameMoveType::EnPassant => 8,
        };
        res as u16
    }

    #[inline(always)]
    pub fn u16_to_mv(mv: u16, game_state: &GameState) -> GameMove {
        let typ = mv & 15;
        let from = ((mv & 0xFC00) >> 10) as usize;
        let from_board = 1u64 << from;
        let to = ((mv & 0x03F0) >> 4) as usize;
        let to_board = 1u64 << to;
        let color_to_move = game_state.color_to_move;
        let enemy_color = 1 - color_to_move;
        let piece_type = if (game_state.pieces[PAWN][color_to_move] & from_board) != 0u64 {
            PieceType::Pawn
        } else if (game_state.pieces[KNIGHT][color_to_move] & from_board) != 0u64 {
            PieceType::Knight
        } else if (game_state.pieces[BISHOP][color_to_move] & from_board) != 0u64 {
            PieceType::Bishop
        } else if (game_state.pieces[ROOK][color_to_move] & from_board) != 0u64 {
            PieceType::Rook
        } else if (game_state.pieces[QUEEN][color_to_move] & from_board) != 0u64 {
            PieceType::Queen
        } else {
            PieceType::King
        };
        if typ == 1 {
            GameMove {
                from,
                to,
                piece_type,
                move_type: GameMoveType::Quiet,
            }
        } else if typ == 2 {
            debug_assert_eq!(piece_type, PieceType::King);
            GameMove {
                from,
                to,
                piece_type,
                move_type: GameMoveType::Castle,
            }
        } else {
            if typ == 8 {
                return GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::EnPassant,
                };
            }
            let captured_piece_type = if (game_state.pieces[PAWN][enemy_color] & to_board) != 0u64 {
                PieceType::Pawn
            } else if (game_state.pieces[KNIGHT][enemy_color] & to_board) != 0u64 {
                PieceType::Knight
            } else if (game_state.pieces[BISHOP][enemy_color] & to_board) != 0u64 {
                PieceType::Bishop
            } else if (game_state.pieces[ROOK][enemy_color] & to_board) != 0u64 {
                PieceType::Rook
            } else if (game_state.pieces[QUEEN][enemy_color] & to_board) != 0u64 {
                PieceType::Queen
            } else {
                PieceType::King
            };
            if typ == 3 {
                GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::Promotion(
                        PieceType::Queen,
                        if captured_piece_type != PieceType::King {
                            Some(captured_piece_type)
                        } else {
                            None
                        },
                    ),
                }
            } else if typ == 4 {
                GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::Promotion(
                        PieceType::Rook,
                        if captured_piece_type != PieceType::King {
                            Some(captured_piece_type)
                        } else {
                            None
                        },
                    ),
                }
            } else if typ == 5 {
                GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::Promotion(
                        PieceType::Bishop,
                        if captured_piece_type != PieceType::King {
                            Some(captured_piece_type)
                        } else {
                            None
                        },
                    ),
                }
            } else if typ == 6 {
                GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::Promotion(
                        PieceType::Knight,
                        if captured_piece_type != PieceType::King {
                            Some(captured_piece_type)
                        } else {
                            None
                        },
                    ),
                }
            } else {
                GameMove {
                    from,
                    to,
                    piece_type,
                    move_type: GameMoveType::Capture(captured_piece_type),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CacheEntry;
    use crate::board_representation::game_state::{GameMove, GameMoveType, GameState, PieceType};
    use crate::move_generation::makemove::make_move;

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
        game_state = make_move(
            &game_state,
            &GameMove {
                from: 23,
                to: 31,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Quiet,
            },
        );
        game_state = make_move(
            &game_state,
            &GameMove {
                from: 50,
                to: 34,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Quiet,
            },
        );
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
