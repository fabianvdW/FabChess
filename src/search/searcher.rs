use super::alphabeta::principal_variation_search;
use super::cache::{Cache, CacheEntry};
use super::history::History;
use super::statistics::SearchStatistics;
use super::timecontrol::{TimeControl, TimeControlInformation};
use super::GameMove;
use super::PrincipalVariation;
use super::MATED_IN_MAX;
use super::MAX_SEARCH_DEPTH;
use super::STANDARD_SCORE;
use crate::board_representation::game_state::{GameState, WHITE};
//use crate::logging::log;
use super::reserved_memory::ReserveMemory;
use crate::board_representation::game_state_attack_container::GameStateAttackContainer;
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen::{generate_moves, MoveList};
use crate::search::reserved_memory::{ReservedAttackContainer, ReservedMoveList};
use crate::search::{CombinedSearchParameters, ScoredPrincipalVariation};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

pub const DEFAULT_THREADS: usize = 1;
pub const MAX_THREADS: usize = 65536;
pub const MIN_THREADS: usize = 1;

#[derive(Copy, Clone)]
pub enum DepthInformation {
    FullySearched,
    CurrentlySearchedBy(usize),
    UnSearched,
}
pub struct InterThreadCommunicationSystem {
    pub threads: usize,
    pub best_pv: Mutex<ScoredPrincipalVariation>,
    pub stable_pv: AtomicBool,
    pub depth_info: Mutex<[DepthInformation; MAX_SEARCH_DEPTH]>,
    pub start_time: Instant,            //Only used for reporting
    pub nodes_searched: Vec<AtomicU64>, // Only used for reporting
    pub nodes_searched_sum: AtomicU64,  // Only used for reporting
    pub seldepth: AtomicUsize,          // Only used for reporting
    pub cache: Arc<Cache>,              //Only used for reporting
}

impl InterThreadCommunicationSystem {
    pub fn new(threads: usize, cache: Arc<Cache>) -> Self {
        let mut nodes_searched = Vec::with_capacity(threads);
        for _ in 0..threads {
            nodes_searched.push(AtomicU64::new(0));
        }
        InterThreadCommunicationSystem {
            threads,
            best_pv: Mutex::new(ScoredPrincipalVariation::default()),
            stable_pv: AtomicBool::new(false),
            depth_info: Mutex::new([DepthInformation::UnSearched; MAX_SEARCH_DEPTH]),
            nodes_searched,
            nodes_searched_sum: AtomicU64::new(0),
            seldepth: AtomicUsize::new(0),
            start_time: Instant::now(),
            cache,
        }
    }

    pub fn get_time_elapsed(&self) -> u64 {
        let now = Instant::now();
        let dur = now.duration_since(self.start_time);
        dur.as_millis() as u64
    }

    pub fn update(&self, thread_id: usize, nodes_searched: u64, seldepth: usize) {
        let curr_seldepth = self.seldepth.load(Ordering::Relaxed);
        self.seldepth
            .store(curr_seldepth.max(seldepth), Ordering::Relaxed);
        let nodes_before = self.nodes_searched[thread_id].load(Ordering::Relaxed);
        self.nodes_searched[thread_id].store(nodes_searched, Ordering::Relaxed);
        self.nodes_searched_sum
            .store(nodes_searched - nodes_before, Ordering::Relaxed)
    }

    pub fn register_pv(&self, pv: &PrincipalVariation, score: i16, depth: usize) {
        let mut curr_best = self.best_pv.lock().unwrap();
        if curr_best.depth < depth || (curr_best.depth == depth && curr_best.score < score) {
            //Update pv stability
            if let Some(other_mv) = curr_best.pv.pv[0] {
                if other_mv == pv.pv[0].unwrap() {
                    self.stable_pv.store(true, Ordering::Relaxed);
                } else {
                    self.stable_pv.store(false, Ordering::Relaxed);
                }
            }
            curr_best.score = score;
            curr_best.depth = depth;
            curr_best.pv = pv.clone();
            //Report to UCI
            let searched_nodes = self.nodes_searched_sum.load(Ordering::Relaxed);
            let elapsed_time = self.get_time_elapsed();
            println!(
                "info depth {} seldepth {} nodes {} nps {} hashfull {:.0} score cp {} pv {}",
                depth,
                self.seldepth.load(Ordering::Relaxed),
                searched_nodes,
                (searched_nodes as f64 / (elapsed_time as f64 / 1000.0)) as u64,
                self.cache.get_status(),
                score,
                pv
            );
        }
    }

    pub fn report_bestmove(&self) {
        println!(
            "bestmove {:?}",
            self.best_pv.lock().unwrap().pv.pv[0]
                .as_ref()
                .expect("Could not unwrap pv for bestmove!")
        );
    }

    pub fn get_next_depth(&self, mut from_depth: usize) -> (usize, bool) {
        from_depth -= 1; //Depth 1 has index 0
        let mut depth_info = self.depth_info.lock().unwrap();
        depth_info[from_depth] = DepthInformation::FullySearched;
        let mut next_depth = from_depth + 1;
        let mut main_thread = false;
        while next_depth <= MAX_SEARCH_DEPTH {
            match depth_info[next_depth] {
                DepthInformation::FullySearched => {
                    next_depth += 1;
                }
                DepthInformation::CurrentlySearchedBy(other_thread) => {
                    if other_thread as f64 >= self.threads as f64 / 2. {
                        next_depth += 1;
                    } else {
                        break;
                    }
                }
                DepthInformation::UnSearched => {
                    main_thread = true;
                    break;
                }
            }
        }
        (next_depth, main_thread)
    }
}
pub struct Thread {
    pub id: usize,
    pub itcs: Arc<InterThreadCommunicationSystem>,
    pub root_plies_played: usize,
    pub history: History,
    pub movelist: ReservedMoveList,
    pub attack_container: ReservedAttackContainer,
    pub killer_moves: [[Option<GameMove>; 2]; MAX_SEARCH_DEPTH],
    pub quiets_tried: [[Option<GameMove>; 128]; MAX_SEARCH_DEPTH],
    pub hh_score: [[[usize; 64]; 64]; 2],
    pub bf_score: [[[usize; 64]; 64]; 2],
    pub history_score: [[[isize; 64]; 64]; 2],
    pub see_buffer: Vec<i16>,
    pub search_statistics: SearchStatistics,
    pub tc: Option<TimeControl>,
    pub time_saved: Option<u64>,
    pub timeout_stop: Arc<AtomicBool>,
    pub current_pv: ScoredPrincipalVariation,
    pub main_thread_in_depth: bool,
}
impl Thread {
    fn new(
        id: usize,
        itcs: Arc<InterThreadCommunicationSystem>,
        root_plies_played: usize,
        history: History,
        tc: Option<TimeControl>,
        time_saved: Option<u64>,
        stop: Arc<AtomicBool>,
    ) -> Self {
        Thread {
            id,
            itcs,
            root_plies_played,
            history,
            movelist: ReservedMoveList::default(),
            attack_container: ReservedAttackContainer::default(),
            killer_moves: [[None; 2]; MAX_SEARCH_DEPTH],
            quiets_tried: [[None; 128]; MAX_SEARCH_DEPTH],
            hh_score: [[[0; 64]; 64]; 2],
            bf_score: [[[1; 64]; 64]; 2],
            history_score: [[[0; 64]; 64]; 2],
            see_buffer: vec![0i16; MAX_SEARCH_DEPTH],
            search_statistics: SearchStatistics::default(),
            tc,
            time_saved,
            timeout_stop: stop,
            current_pv: ScoredPrincipalVariation::default(),
            main_thread_in_depth: false,
        }
    }

    fn search(&mut self, max_depth: i16, state: GameState) {
        println!(
            "info String Thread {} starting the search of state!",
            self.id
        );
    }
}

pub fn search_move(
    max_depth: i16,
    game_state: GameState,
    history: Vec<GameState>,
    stop_ref: Arc<AtomicBool>,
    cache: Arc<Cache>,
    saved_time: Arc<AtomicU64>,
    _last_score: i16,
    threads: usize,
    tc: TimeControl,
) -> Option<i16> {
    let time_saved_before = saved_time.load(Ordering::Relaxed);
    //Step 1. Check how many legal moves there are
    let mut movelist = MoveList::default();
    generate_moves(
        &game_state,
        false,
        &mut movelist,
        &mut GameStateAttackContainer::from_state(&game_state),
    );
    if movelist.counter == 0 {
        panic!("The root position given does not have any legal move!");
    } else if movelist.counter == 1 {
        println!(
            "bestmove {:?}",
            movelist.move_list[0]
                .as_ref()
                .expect("Can't unwrap move although there is one")
        );

        let new_timesaved: u64 =
            (time_saved_before as i64 + tc.time_saved(0, time_saved_before)).max(0) as u64;
        saved_time.store(new_timesaved, Ordering::Relaxed);
        return None;
    }

    //Step 2. Prepare threads
    let mut hist: History = History::default();
    let mut relevant_hashes: Vec<u64> = Vec::with_capacity(100);
    for gs in history.iter().rev() {
        relevant_hashes.push(gs.hash);
        if gs.half_moves == 0 {
            break;
        }
    }
    for hashes in relevant_hashes.iter().rev() {
        hist.push(*hashes, false);
    }
    let root_plies_played = (game_state.full_moves - 1) * 2 + game_state.color_to_move;
    let itcs = Arc::new(InterThreadCommunicationSystem::new(
        threads,
        Arc::clone(&cache),
    ));

    //the only special thing about the main thread is that it takes care of the timecontrol
    let mut main_thread = Thread::new(
        0,
        Arc::clone(&itcs),
        root_plies_played,
        hist.clone(),
        Some(tc.clone()),
        Some(time_saved_before),
        Arc::clone(&stop_ref),
    );
    let mut childs = Vec::new();
    for id in 1..threads {
        let itcs_clone = Arc::clone(&itcs);
        let hist_clone = hist.clone();
        let state_clone = game_state.clone();
        let stop_clone = Arc::clone(&stop_ref);
        childs.push(thread::spawn(move || {
            let mut thread = Thread::new(
                id,
                itcs_clone,
                root_plies_played,
                hist_clone,
                None,
                None,
                stop_clone,
            );
            thread.search(max_depth, state_clone);
        }));
    }
    main_thread.search(max_depth, game_state);
    for child in childs {
        child.join().expect("Couldn't join thread");
    }
    //Report to UCI
    itcs.report_bestmove();
    //Store new saved time
    let elapsed_time = itcs.get_time_elapsed();
    let new_timesaved: u64 =
        (time_saved_before as i64 + tc.time_saved(elapsed_time, time_saved_before)).max(0) as u64;
    saved_time.store(new_timesaved, Ordering::Relaxed);
    //And return
    let best_pv = itcs.best_pv.lock().unwrap();
    Some(best_pv.score)
}
pub struct SearchUtils<'a> {
    pub root_pliesplayed: usize,
    pub search: &'a mut Search,
    pub history: &'a mut History,
    pub stop: &'a Arc<AtomicBool>,
    pub cache: &'a Cache,
    pub thread_memory: &'a mut ReserveMemory,
}

impl<'a> SearchUtils<'a> {
    pub fn new(
        root_pliesplayed: usize,
        search: &'a mut Search,
        history: &'a mut History,
        stop: &'a Arc<AtomicBool>,
        cache: &'a Cache,
        thread_memory: &'a mut ReserveMemory,
    ) -> Self {
        SearchUtils {
            root_pliesplayed,
            search,
            history,
            stop,
            cache,
            thread_memory,
        }
    }
}

pub struct Search {
    pub principal_variation: [Option<CacheEntry>; MAX_SEARCH_DEPTH],
    pub pv_table: Vec<PrincipalVariation>,
    pub killer_moves: [[Option<GameMove>; 2]; MAX_SEARCH_DEPTH],
    pub quiets_tried: [[Option<GameMove>; 128]; MAX_SEARCH_DEPTH],
    pub hh_score: [[[usize; 64]; 64]; 2],
    pub bf_score: [[[usize; 64]; 64]; 2],
    pub history_score: [[[isize; 64]; 64]; 2],
    pub see_buffer: Vec<i16>,
    pub search_statistics: SearchStatistics,
    pub tc: TimeControl,
    pub tc_information: TimeControlInformation,
    pub stop: bool,
}

impl Search {
    pub fn new(tc: TimeControl, tc_information: TimeControlInformation) -> Search {
        let mut pv_table = Vec::with_capacity(MAX_SEARCH_DEPTH);
        for i in 0..MAX_SEARCH_DEPTH {
            pv_table.push(PrincipalVariation::new(MAX_SEARCH_DEPTH - i));
        }
        Search {
            principal_variation: [None; MAX_SEARCH_DEPTH],
            pv_table,
            search_statistics: SearchStatistics::default(),
            killer_moves: [[None; 2]; MAX_SEARCH_DEPTH],
            quiets_tried: [[None; 128]; MAX_SEARCH_DEPTH],
            hh_score: [[[0; 64]; 64]; 2],
            bf_score: [[[1; 64]; 64]; 2],
            history_score: [[[0; 64]; 64]; 2],
            see_buffer: vec![0i16; MAX_SEARCH_DEPTH],
            tc,
            tc_information,
            stop: false,
        }
    }

    pub fn replace_pv(&mut self, game_state: &GameState, depth: i16, pv_score: i16) {
        let mut pv_stack = Vec::with_capacity(depth as usize);
        let mut index = 0;
        while let Some(pair) = self.pv_table[0].pv[index].as_ref() {
            if index == 0 {
                pv_stack.push(make_move(&game_state, &pair));
            } else {
                pv_stack.push(make_move(&pv_stack[index - 1], &pair));
            }
            index += 1;
        }
        index = 0;
        while let Some(pair) = self.pv_table[0].pv[index].as_ref() {
            let state = if index == 0 {
                &game_state
            } else {
                &pv_stack[index - 1]
            };
            self.principal_variation[index] = Some(CacheEntry::new(
                state,
                depth - index as i16,
                pv_score,
                false,
                false,
                &pair,
                None,
                true,
            ));
            index += 1;
        }
    }
    pub fn search(
        &mut self,
        depth: i16,
        game_state: GameState,
        history: Vec<GameState>,
        stop_ref: Arc<AtomicBool>,
        cache: Arc<Cache>,
        saved_time: Arc<AtomicU64>,
        _last_score: i16,
    ) -> i16 {
        let root_plies_played = (game_state.full_moves - 1) * 2 + game_state.color_to_move;
        let mut hist: History = History::default();
        let mut relevant_hashes: Vec<u64> = Vec::with_capacity(100);
        for gs in history.iter().rev() {
            relevant_hashes.push(gs.hash);
            if gs.half_moves == 0 {
                break;
            }
        }
        for hashes in relevant_hashes.iter().rev() {
            hist.push(*hashes, false);
        }

        self.search_statistics = SearchStatistics::default();
        let mut reserved_memory = ReserveMemory::default();
        let mut best_pv_score = STANDARD_SCORE;

        for d in 1..=depth {
            let mut pv_score;
            if d == 1 {
                let mut searchutils = SearchUtils::new(
                    root_plies_played,
                    self,
                    &mut hist,
                    &stop_ref,
                    &cache,
                    &mut reserved_memory,
                );
                pv_score = principal_variation_search(
                    CombinedSearchParameters::from(
                        -16000,
                        16000,
                        d,
                        &game_state,
                        if game_state.color_to_move == WHITE {
                            1
                        } else {
                            -1
                        },
                        0,
                    ),
                    &mut searchutils,
                );
            } else {
                //Aspiration Window
                let mut delta = 40;
                let mut alpha = best_pv_score - delta;
                let mut beta = best_pv_score + delta;
                loop {
                    let mut searchutils = SearchUtils::new(
                        root_plies_played,
                        self,
                        &mut hist,
                        &stop_ref,
                        &cache,
                        &mut reserved_memory,
                    );
                    pv_score = principal_variation_search(
                        CombinedSearchParameters::from(
                            alpha,
                            beta,
                            d,
                            &game_state,
                            if game_state.color_to_move == WHITE {
                                1
                            } else {
                                -1
                            },
                            0,
                        ),
                        &mut searchutils,
                    );
                    if self.stop {
                        break;
                    }
                    /*if (pv_score - last_score).abs() > 150 && last_score.abs() < 600 {
                        self.tc_information.high_score_diff = true;
                    } else {
                        self.tc_information.high_score_diff = false;
                    }*/

                    if pv_score > alpha && pv_score < beta {
                        break;
                    }
                    //Else put pv in principal_variation table
                    //self.replace_pv(&game_state, depth, pv_score);

                    if pv_score <= alpha {
                        if alpha < -10000 || pv_score < MATED_IN_MAX {
                            alpha = -16000;
                            beta = 16000;
                        } else {
                            alpha -= delta;
                        }
                    }
                    if pv_score >= beta {
                        if beta > 10000 || pv_score > -MATED_IN_MAX {
                            beta = 16000;
                            alpha = -16000;
                        } else {
                            beta += delta;
                        }
                    }
                    delta = (f64::from(delta) * 1.5) as i16;
                }
            }
            if self.stop {
                break;
            }

            let nps = self.search_statistics.getnps();
            println!(
                "{}",
                format!(
                    "info depth {} seldepth {} nodes {} nps {} time {} hashfull {:.0} score cp {} multipv 1 pv {}",
                    d,
                    self.search_statistics.seldepth,
                    self.search_statistics.nodes_searched,
                    nps,
                    self.search_statistics.time_elapsed,
                    cache.full.load(std::sync::atomic::Ordering::Relaxed) as f64
                        / cache.entries as f64
                        * 1000.,
                    pv_score,
                    self.pv_table[0]
                )
            );
            //println!("{}", self.search_statistics);

            //Compare old pv to new pv
            if let Some(ce) = self.principal_variation[0].as_ref() {
                let old_mv: GameMove = CacheEntry::u16_to_mv(ce.mv, &game_state);
                let new_mv: &GameMove = self.pv_table[0].pv[0]
                    .as_ref()
                    .expect("Couldn't unwrap first move of new pv");
                if old_mv == (*new_mv) {
                    self.tc_information.stable_pv = true;
                } else {
                    self.tc_information.stable_pv = false;
                }
            }
            //println!("{}", self.search_statistics);
            //Set PV in table
            self.replace_pv(&game_state, depth, pv_score);
            best_pv_score = pv_score;
        }
        self.search_statistics.refresh_time_elapsed();
        /*log(&format!(
            "\nFinished calculating game_state with plies: {}\n",
            game_state.full_moves
        ));
        log(&format!("{}\n", self.tc.to_string(&self.tc_information)));
        log(&format!(
            "Time elapsed: {}\n",
            self.search_statistics.time_elapsed
        ));
        log(&format!(
            "Time saved in this: {}\n",
            self.tc.time_saved(
                self.search_statistics.time_elapsed,
                self.tc_information.time_saved
            )
        ));*/
        let mut new_timesaved: i64 = self.tc_information.time_saved as i64
            + self.tc.time_saved(
                self.search_statistics.time_elapsed,
                self.tc_information.time_saved,
            );
        new_timesaved = new_timesaved.max(0);
        saved_time.store(new_timesaved as u64, Ordering::Relaxed);
        /*log(&format!(
            "New total time saved: {}\n",
            saved_time.load(Ordering::Relaxed)
        ));*/
        best_pv_score
    }
}
