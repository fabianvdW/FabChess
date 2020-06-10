use crate::board_representation::game_state::{
    GameMove, GameMoveType, GameState, PieceType, PIECE_TYPES,
};
use crate::search::searcher::Thread;
use crate::search::{CombinedSearchParameters, SearchInstruction, MATED_IN_MAX};
use std::cell::UnsafeCell;

pub const DEFAULT_HASH_SIZE: usize = 256;
//IN MB
pub const MIN_HASH_SIZE: usize = 0;
//IN MB
pub const MAX_HASH_SIZE: usize = 131_072; //IN MB

pub struct Cache {
    pub entries: usize,
    pub buckets: usize,
    pub cache: UnsafeCell<Vec<CacheBucket>>,
}

unsafe impl std::marker::Sync for Cache {}

impl Cache {
    pub fn score_to_tt_score(score: i16, current_depth: i16) -> i16 {
        if score.abs() >= MATED_IN_MAX.abs() {
            if score > -MATED_IN_MAX {
                score + current_depth
            } else {
                score - current_depth
            }
        } else {
            score
        }
    }
    pub fn score_from_tt_score(score: i16, current_depth: i16) -> i16 {
        if score.abs() >= MATED_IN_MAX.abs() {
            if score > -MATED_IN_MAX {
                score - current_depth
            } else {
                score + current_depth
            }
        } else {
            score
        }
    }

    pub fn with_size_threaded(mb_size: usize, num_threads: usize) -> Self {
        let buckets = 1024 * 1024 * mb_size / 64;
        let entries = buckets * 5;
        let cache = UnsafeCell::new(Cache::get_init_cache(buckets, num_threads));
        Cache {
            entries,
            buckets,
            cache,
        }
    }

    fn get_init_cache(buckets: usize, num_threads: usize) -> Vec<CacheBucket> {
        // The cache may be large so initialize it using multiple threads, if possible.
        // This is relevant for events like TCEC, where huge memory is allocated (like 64GB),
        // and a startup time of >1min is unacceptable.
        let mut cache_vec: Vec<CacheBucket> = Vec::with_capacity(buckets);
        unsafe {
            cache_vec.set_len(buckets);
            let chunksize = (buckets + num_threads - 1) / num_threads;

            let mut ptr = cache_vec.as_mut_ptr();
            let mut handles = Vec::new();

            for t in 0..num_threads {
                // The last chunk may be shorter.
                let this_chunk = chunksize.min(buckets - t * chunksize);

                // circumvent the fact that raw pointers are not Send
                let w = PtrWrapper { p: ptr };
                handles.push(std::thread::spawn(move || {
                    let mut inner_ptr = w.p;
                    for _ in 0..this_chunk {
                        *inner_ptr = CacheBucket::default();
                        inner_ptr = inner_ptr.offset(1);
                    }
                }));
                ptr = ptr.add(chunksize);
            }

            for handle in handles {
                handle
                    .join()
                    .expect("Could not unwrap handle while initializing the cache!");
            }
        }
        cache_vec
    }

    pub fn fill_status(&self) -> usize {
        if self.entries < 1000 {
            return 1000;
        }
        //Count bottom 500 entries
        let mut counted_entries = 0;
        let mut full = 0;

        let mut index = 0;
        while index < unsafe { (&*self.cache.get()).len() } && counted_entries < 500 {
            let bucket = unsafe { (&*self.cache.get()).get(index).unwrap() };
            index += 1;
            full += bucket.fill_status();
            counted_entries += 5;
        }
        //Count upper 500 entries
        let mut index = unsafe { (&*self.cache.get()).len() - 1 };
        while counted_entries < 1000 {
            let bucket = unsafe { (&*self.cache.get()).get(index).unwrap() };
            debug_assert!(index > 0);
            full += bucket.fill_status();
            counted_entries += 5;
            if index == 0 {
                break;
            }
            index -= 1;
        }
        (full as f64 / counted_entries as f64 * 1000.0) as usize
    }

    pub fn clear_threaded(&self, num_threads: usize) {
        unsafe {
            *self.cache.get() = Cache::get_init_cache(self.buckets, num_threads);
        }
    }

    pub fn insert(
        &self,
        p: &CombinedSearchParameters,
        thread: &Thread,
        mv: GameMove,
        score: i16,
        original_alpha: i16,
    ) {
        if self.entries == 0 {
            return;
        }
        let index = p.game_state.get_hash() as usize % self.buckets;
        unsafe {
            (&mut *self.cache.get())
                .get_unchecked_mut(index)
                .replace_entry(p, thread, mv, score, original_alpha);
        };
    }

    pub fn lookup(
        &self,
        p: &CombinedSearchParameters,
        tt_entry: &mut Option<CacheEntry>,
    ) -> SearchInstruction {
        if self.entries == 0 {
            return SearchInstruction::ContinueSearching;
        }
        let ce = unsafe {
            (&mut *self.cache.get())
                .get_mut(p.game_state.get_hash() as usize % self.buckets)
                .unwrap()
                .probe(p.game_state.get_hash())
        };
        if let Some(mut ce) = ce {
            ce.score = Cache::score_from_tt_score(ce.score, p.current_depth as i16);
            *tt_entry = Some(ce);
            if ce.depth >= p.depth_left as i8
                && (p.beta - p.alpha <= 1 || p.depth_left <= 0)
                && (ce.exact_bound()
                    || ce.lower_bound() && ce.score >= p.beta
                    || ce.upper_bound() && ce.score <= p.alpha)
            {
                return SearchInstruction::StopSearching(ce.score);
            }
        }
        SearchInstruction::ContinueSearching
    }
}

#[repr(align(64))]
#[derive(Copy, Clone)]
pub struct CacheBucket([CacheEntry; 5]);

impl CacheBucket {
    pub fn replace_entry(
        &mut self,
        p: &CombinedSearchParameters,
        thread: &Thread,
        mv: GameMove,
        score: i16,
        original_alpha: i16,
    ) -> bool {
        let lower_bound = score >= p.beta;
        let upper_bound = score <= original_alpha;
        let score = Cache::score_to_tt_score(score, p.current_depth as i16);
        let write_entry = |cache_entry: &mut CacheEntry| {
            cache_entry.write(
                p.game_state.get_hash(),
                p.depth_left,
                score,
                upper_bound,
                lower_bound,
                mv,
            );
        };
        for i in 0..self.0.len() {
            if self.0[i].is_invalid() {
                write_entry(&mut self.0[i]);
                return true;
            } else if self.0[i].validate_hash(p.game_state.get_hash()) {
                if self.0[i].depth < p.depth_left as i8
                    || self.0[i].depth == p.depth_left as i8
                        && (self.0[i].upper_bound() && score < self.0[i].score
                            || self.0[i].lower_bound() && score < self.0[i].score)
                {}
                write_entry(&mut self.0[i]);
                return true;
            }
        }
        for i in 0..self.0.len() {
            if self.0[i].depth < (p.depth_left as usize + i) as i8 {
                write_entry(&mut self.0[i]);
                return true;
            }
        }
        false
    }

    pub fn probe(&mut self, hash: u64) -> Option<CacheEntry> {
        if hash == 0u64 {
            return None;
        }
        if self.0[0].validate_hash(hash) {
            return Some(self.0[0]);
        } else if self.0[1].validate_hash(hash) {
            return Some(self.0[1]);
        } else if self.0[2].validate_hash(hash) {
            return Some(self.0[2]);
        } else if self.0[3].validate_hash(hash) {
            return Some(self.0[3]);
        } else if self.0[4].validate_hash(hash) {
            return Some(self.0[4]);
        }
        None
    }
    pub fn swap_entries(&mut self, a: usize, b: usize) {
        let at_a = self.0[a];
        self.0[a] = self.0[b];
        self.0[b] = at_a;
    }

    pub fn fill_status(&self) -> usize {
        let mut res = 0;
        for entry in self.0.iter() {
            if !entry.is_invalid() {
                res += 1;
            }
        }
        res
    }
}

impl Default for CacheBucket {
    fn default() -> Self {
        CacheBucket([CacheEntry::invalid(); 5])
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CacheEntry {
    pub flags: u8,
    pub depth: i8,
    pub score: i16,
    pub upper_hash: u32,
    pub lower_hash: u16,
    pub mv: u16,
}

impl CacheEntry {
    pub fn lower_bound(&self) -> bool {
        self.flags & 1 > 0
    }
    pub fn upper_bound(&self) -> bool {
        self.flags & 2 > 0
    }
    pub fn exact_bound(&self) -> bool {
        !self.lower_bound() && !self.upper_bound()
    }

    //We assume at least 16 bits are used for indexing
    //That would mean 65536 buckets or: 65536 *64 bytes -> >= 4MB.
    //That is a pretty fair assumption to make, especially sinc we
    //fully valide tt moves anyway, and this allows us to fit
    //5 entries into a bucket.
    pub fn validate_hash(&self, hash: u64) -> bool {
        (self.upper_hash as u64) == (hash >> 32)
            && (((self.lower_hash ^ self.mv) as u64) << 16) == (hash & 0xFFFF_0000)
    }
    //I know this is not idiomatic, but it saves memory...
    pub fn is_invalid(&self) -> bool {
        self.mv == 0u16
    }

    pub fn invalid() -> CacheEntry {
        CacheEntry {
            upper_hash: 0,
            lower_hash: 0,
            depth: 0,
            score: 0,
            mv: 0,
            flags: 0,
        }
    }

    pub fn write(
        &mut self,
        hash: u64,
        depth: i16,
        score: i16,
        upper_bound: bool,
        lower_bound: bool,
        mv: GameMove,
    ) {
        let mv = CacheEntry::mv_to_u16(mv);
        self.upper_hash = (hash >> 32) as u32;
        self.lower_hash = ((hash & 0xFFFF_0000) >> 16) as u16 ^ mv;
        self.depth = depth as i8;
        self.score = score;
        self.flags = 0u8;
        self.flags |= lower_bound as u8;
        self.flags |= (upper_bound as u8) << 1;
        self.mv = mv;
    }

    #[inline(always)]
    pub fn mv_to_u16(mv: GameMove) -> u16 {
        let mut res = 0;
        res |= (mv.from as usize) << 10;
        res |= (mv.to as usize) << 4;
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
        let from = ((mv & 0xFC00) >> 10) as u8;
        let from_board = 1u64 << from;
        let to = ((mv & 0x03F0) >> 4) as u8;
        let to_board = 1u64 << to;
        let color_to_move = game_state.get_color_to_move();
        let enemy_color = 1 - color_to_move;
        let mut piece_type = PieceType::Pawn;
        for pt in PIECE_TYPES.iter() {
            if game_state.get_piece(*pt, color_to_move) & from_board > 0 {
                piece_type = *pt;
            }
        }
        if typ == 1 {
            GameMove {
                from,
                to,
                piece_type,
                move_type: GameMoveType::Quiet,
            }
        } else if typ == 2 {
            //debug_assert_eq!(piece_type, PieceType::King); //We literally expect TRASH in here.
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
            let mut captured_piece_type = PieceType::King;
            for pt in PIECE_TYPES.iter() {
                if game_state.get_piece(*pt, enemy_color) & to_board > 0 {
                    captured_piece_type = *pt;
                }
            }
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

struct PtrWrapper<T> {
    pub p: *mut T,
}

unsafe impl<T> Send for PtrWrapper<T> {}

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
            let h3h4u16 = CacheEntry::mv_to_u16(h3h4);
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
            let h3g4u16 = CacheEntry::mv_to_u16(h3g4);
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
            let e1c1u16 = CacheEntry::mv_to_u16(e1c1);
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
            let e1g1u16 = CacheEntry::mv_to_u16(e1g1);
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
            let e7e8qu16 = CacheEntry::mv_to_u16(e7e8q);
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
            let e7e8ru16 = CacheEntry::mv_to_u16(e7e8r);
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
            let e7e8bu16 = CacheEntry::mv_to_u16(e7e8b);
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
            let e7e8nu16 = CacheEntry::mv_to_u16(e7e8n);
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
            let e7e8qu16 = CacheEntry::mv_to_u16(e7e8q);
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
            let e7e8ru16 = CacheEntry::mv_to_u16(e7e8r);
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
            let e7e8bu16 = CacheEntry::mv_to_u16(e7e8b);
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
            let e7e8nu16 = CacheEntry::mv_to_u16(e7e8n);
            let e7e8nres = CacheEntry::u16_to_mv(e7e8nu16, &game_state);
            assert_eq!(e7e8nres.from, e7e8n.from);
            assert_eq!(e7e8nres.to, e7e8n.to);
            assert_eq!(e7e8nres.move_type, e7e8n.move_type);
            assert_eq!(e7e8nres.piece_type, e7e8n.piece_type);
        }
        game_state = make_move(
            &game_state,
            GameMove {
                from: 23,
                to: 31,
                piece_type: PieceType::Pawn,
                move_type: GameMoveType::Quiet,
            },
        );
        game_state = make_move(
            &game_state,
            GameMove {
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
            let d5d6u16 = CacheEntry::mv_to_u16(d5d6);
            let d5d6res = CacheEntry::u16_to_mv(d5d6u16, &game_state);
            assert_eq!(d5d6res.from, d5d6.from);
            assert_eq!(d5d6res.to, d5d6.to);
            assert_eq!(d5d6res.move_type, d5d6.move_type);
            assert_eq!(d5d6res.piece_type, d5d6.piece_type);
        }
    }
}
