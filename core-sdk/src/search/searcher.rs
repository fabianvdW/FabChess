use super::alphabeta::principal_variation_search;
use super::cache::Cache;
use super::history::History;
use super::statistics::SearchStatistics;
use super::timecontrol::TimeControl;
use super::GameMove;
use super::PrincipalVariation;
use super::MATED_IN_MAX;
use super::MAX_SEARCH_DEPTH;
use crate::board_representation::game_state::{GameState, WHITE};
//use crate::logging::log;
use crate::move_generation::makemove::make_move;
use crate::move_generation::movegen::{generate_moves, MoveList};
use crate::search::reserved_memory::ReservedMoveList;
use crate::search::{CombinedSearchParameters, ScoredPrincipalVariation, MATE_SCORE};
use crate::UCIOptions;
use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;

pub const DEFAULT_SKIP_RATIO: usize = 2;
pub const MIN_SKIP_RATIO: usize = 1;
pub const MAX_SKIP_RATIO: usize = 1024;

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
    pub uci_options: UnsafeCell<UCIOptions>,
    pub best_pv: Mutex<ScoredPrincipalVariation>,
    pub stable_pv: AtomicBool,
    pub depth_info: Mutex<[DepthInformation; MAX_SEARCH_DEPTH]>,
    pub start_time: RwLock<Instant>, //Only used for reporting
    pub nodes_searched: UnsafeCell<Vec<AtomicU64>>, // Only used for reporting
    pub seldepth: AtomicUsize,       // Only used for reporting
    pub cache: UnsafeCell<Cache>,    //Only used for reporting
    pub cache_status: AtomicUsize,
    pub last_cache_status: Mutex<Option<Instant>>,
    pub timeout_flag: RwLock<bool>,
    pub saved_time: AtomicU64,
    pub tx: RwLock<Vec<Sender<ThreadInstruction>>>,
    rx_f: Receiver<()>,
    tx_f: Sender<()>,
}
impl Default for InterThreadCommunicationSystem {
    fn default() -> Self {
        let (tx_f, rx_f) = channel();
        InterThreadCommunicationSystem {
            uci_options: UnsafeCell::new(UCIOptions::default()),
            best_pv: Mutex::new(ScoredPrincipalVariation::default()),
            stable_pv: AtomicBool::new(false),
            depth_info: Mutex::new([DepthInformation::UnSearched; MAX_SEARCH_DEPTH]),
            nodes_searched: UnsafeCell::new(Vec::new()),
            seldepth: AtomicUsize::new(0),
            start_time: RwLock::new(Instant::now()),
            last_cache_status: Mutex::new(None),
            cache_status: AtomicUsize::new(0),
            cache: UnsafeCell::new(Cache::with_size_threaded(0, 1)),
            timeout_flag: RwLock::new(false),
            saved_time: AtomicU64::new(0u64),
            tx: RwLock::new(Vec::new()),
            rx_f,
            tx_f,
        }
    }
}
impl InterThreadCommunicationSystem {
    pub fn cache(&self) -> &mut Cache {
        unsafe { self.cache.get().as_mut().unwrap() }
    }
    pub fn uci_options(&self) -> &mut UCIOptions {
        unsafe { self.uci_options.get().as_mut().unwrap() }
    }
    pub fn nodes_searched(&self) -> &mut Vec<AtomicU64> {
        unsafe { self.nodes_searched.get().as_mut().unwrap() }
    }

    pub fn update_thread_count(
        itcs: &Arc<InterThreadCommunicationSystem>,
        new_thread_count: usize,
    ) {
        for tx in itcs.tx.read().unwrap().iter() {
            tx.send(ThreadInstruction::Exit)
                .expect("couldn't send exit flag");
        }
        for _ in 0..itcs.tx.read().unwrap().len() {
            itcs.rx_f.recv().expect("Couldn't receive exit flag!")
        }
        itcs.uci_options().threads = new_thread_count;
        let itcs_tx = &mut *itcs.tx.write().unwrap();
        let itcs_nodes_searched = itcs.nodes_searched();
        *itcs_tx = Vec::with_capacity(new_thread_count);
        *itcs_nodes_searched = Vec::with_capacity(new_thread_count);
        for id in 0..new_thread_count {
            itcs_nodes_searched.push(AtomicU64::new(0));
            let (tx, rx) = channel();
            itcs_tx.push(tx);
            let tx_f = itcs.tx_f.clone();
            let self_arc = Arc::clone(&itcs);
            thread::Builder::new()
                .stack_size(12 * 1024 * 1024)
                .spawn(move || {
                    let mut thread = Thread::new(id, self_arc, rx, tx_f);
                    thread.run();
                })
                .expect("Could not build thread");
        }
    }

    pub fn get_time_elapsed(&self) -> u64 {
        let now = Instant::now();
        let dur = now.duration_since(*self.start_time.read().unwrap());
        dur.as_millis() as u64
    }

    pub fn update(&self, thread_id: usize, nodes_searched: u64, seldepth: usize) {
        let curr_seldepth = self.seldepth.load(Ordering::Relaxed);
        self.seldepth
            .store(curr_seldepth.max(seldepth), Ordering::Relaxed);
        self.nodes_searched()[thread_id].store(nodes_searched, Ordering::Relaxed);
    }

    pub fn get_nodes_sum(&self) -> u64 {
        self.nodes_searched()
            .iter()
            .map(|x| x.load(Ordering::Relaxed))
            .sum()
    }

    pub fn register_pv(&self, scored_pv: &ScoredPrincipalVariation, no_fail: bool) {
        let mut curr_best = self.best_pv.lock().unwrap();
        self.stable_pv.store(false, Ordering::Relaxed);
        //Update pv stability
        if let Some(other_mv) = curr_best.pv.pv[0] {
            if other_mv == scored_pv.pv.pv[0].unwrap() && no_fail {
                self.stable_pv.store(true, Ordering::Relaxed);
            }
        }
        if curr_best.depth < scored_pv.depth
            || (curr_best.depth == scored_pv.depth && curr_best.score < scored_pv.score)
        {
            if no_fail {
                *curr_best = scored_pv.clone();
            }
            //Report to UCI
            let searched_nodes: u64 = self.get_nodes_sum();
            let elapsed_time = self.get_time_elapsed();
            let mut cache_status = self.last_cache_status.lock().unwrap();
            let fill_status = if cache_status.is_none()
                || Instant::now()
                    .duration_since(cache_status.unwrap())
                    .as_millis()
                    > 200
            {
                *cache_status = Some(Instant::now());
                self.cache_status
                    .store(self.cache().fill_status(), Ordering::Relaxed);
                self.cache_status.load(Ordering::Relaxed)
            } else {
                self.cache_status.load(Ordering::Relaxed)
            };
            let score_string = if cfg!(feature = "avoid-adj") {
                let score = scored_pv.score.min(200).max(-200);
                let score = if score.abs() < 10 { 25 } else { score };
                format!("score cp {}", score)
            } else if scored_pv.score.abs() > MATE_SCORE - 200 {
                let dtm = if scored_pv.score > 0 {
                    (MATE_SCORE - scored_pv.score) / 2 + 1
                } else {
                    (-MATE_SCORE - scored_pv.score) / 2
                };
                format!("score mate {}", dtm)
            } else {
                format!("score cp {}", scored_pv.score)
            };
            println!(
                "info depth {} seldepth {} nodes {} nps {} hashfull {:.0} time {} {} pv {}",
                scored_pv.depth,
                self.seldepth.load(Ordering::Relaxed),
                searched_nodes,
                (searched_nodes as f64 / (elapsed_time.max(1) as f64 / 1000.0)) as u64,
                fill_status,
                self.get_time_elapsed(),
                score_string,
                scored_pv.pv
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
        if from_depth == 0 {
            return (1, true);
        }
        from_depth -= 1; //Depth 1 has index 0
        let mut depth_info = self.depth_info.lock().unwrap();
        depth_info[from_depth] = DepthInformation::FullySearched;
        let mut next_depth = from_depth + 1;
        let mut main_thread = false;
        while next_depth < MAX_SEARCH_DEPTH {
            match depth_info[next_depth] {
                DepthInformation::FullySearched => {
                    next_depth += 1;
                }
                DepthInformation::CurrentlySearchedBy(other_thread) => {
                    if other_thread as f64
                        >= self.uci_options().threads as f64 / self.uci_options().skip_ratio as f64
                    {
                        next_depth += 1;
                    } else {
                        depth_info[next_depth] =
                            DepthInformation::CurrentlySearchedBy(other_thread + 1);
                        break;
                    }
                }
                DepthInformation::UnSearched => {
                    main_thread = true;
                    depth_info[next_depth] = DepthInformation::CurrentlySearchedBy(1);
                    break;
                }
            }
        }

        (next_depth + 1, main_thread)
    }
}
unsafe impl std::marker::Sync for InterThreadCommunicationSystem {}
pub enum ThreadInstruction {
    Exit,
    StartSearch(i16, GameState, TimeControl, History, u64),
}

pub struct Thread {
    pub id: usize,
    pub itcs: Arc<InterThreadCommunicationSystem>,
    pub root_plies_played: usize,
    pub history: History,
    pub movelist: ReservedMoveList,
    pub pv_table: Vec<PrincipalVariation>,
    pub killer_moves: [[Option<GameMove>; 2]; MAX_SEARCH_DEPTH],
    pub quiets_tried: [[Option<GameMove>; 128]; MAX_SEARCH_DEPTH],
    pub hh_score: [[[usize; 64]; 64]; 2],
    pub bf_score: [[[usize; 64]; 64]; 2],
    pub history_score: [[[isize; 64]; 64]; 2],
    pub see_buffer: Vec<i16>,
    pub search_statistics: SearchStatistics,
    pub tc: TimeControl, //Only thread 0 takes care of Timecontrol though
    pub time_saved: u64,
    pub self_stop: bool, //This is set when timeout_stop is set(timeout_stop isn't always polled)
    pub current_pv: ScoredPrincipalVariation,
    pub pv_applicable: Vec<u64>, //Hashes of gamestates the pv plays along
    pub main_thread_in_depth: bool,
    rx: Receiver<ThreadInstruction>,
    tx: Sender<()>,
}

impl Thread {
    pub fn replace_current_pv(
        &mut self,
        root: &GameState,
        scored_pv: ScoredPrincipalVariation,
        no_fail: bool,
    ) {
        self.itcs.register_pv(&scored_pv, no_fail);
        self.current_pv = scored_pv;
        self.pv_applicable.clear();
        self.pv_applicable.push(root.get_hash());
        let mut next_state = None;
        for mv in self.current_pv.pv.pv.iter() {
            if let Some(mv) = mv {
                if next_state.is_none() {
                    next_state = Some(make_move(root, *mv));
                } else {
                    next_state = Some(make_move(next_state.as_ref().unwrap(), *mv));
                }
                self.pv_applicable
                    .push(next_state.as_ref().unwrap().get_hash());
            } else {
                break;
            }
        }
    }
    fn new(
        id: usize,
        itcs: Arc<InterThreadCommunicationSystem>,
        rx: Receiver<ThreadInstruction>,
        tx: Sender<()>,
    ) -> Self {
        let mut pv_table = Vec::with_capacity(MAX_SEARCH_DEPTH);
        for i in 0..MAX_SEARCH_DEPTH {
            pv_table.push(PrincipalVariation::new(MAX_SEARCH_DEPTH - i));
        }
        Thread {
            id,
            itcs,
            root_plies_played: 0,
            history: History::default(),
            movelist: ReservedMoveList::default(),
            pv_table,
            killer_moves: [[None; 2]; MAX_SEARCH_DEPTH],
            quiets_tried: [[None; 128]; MAX_SEARCH_DEPTH],
            hh_score: [[[0; 64]; 64]; 2],
            bf_score: [[[1; 64]; 64]; 2],
            history_score: [[[0; 64]; 64]; 2],
            see_buffer: vec![0i16; MAX_SEARCH_DEPTH],
            search_statistics: SearchStatistics::default(),
            tc: TimeControl::MoveTime(0u64),
            time_saved: 0u64,
            self_stop: false,
            current_pv: ScoredPrincipalVariation::default(),
            pv_applicable: Vec::with_capacity(MAX_SEARCH_DEPTH),
            main_thread_in_depth: false,
            rx,
            tx,
        }
    }

    fn run(&mut self) {
        loop {
            let msg: ThreadInstruction = self.rx.recv().unwrap();
            match msg {
                ThreadInstruction::Exit => {
                    self.tx.send(()).expect("Error sending exit flag!");
                    break;
                }
                ThreadInstruction::StartSearch(max_depth, state, tc, history, time_saved) => {
                    self.root_plies_played =
                        (state.get_full_moves() - 1) * 2 + state.get_color_to_move();
                    self.history = history;
                    self.time_saved = time_saved;
                    self.pv_applicable.clear();
                    self.current_pv = ScoredPrincipalVariation::default();
                    self.main_thread_in_depth = false;
                    self.killer_moves = [[None; 2]; MAX_SEARCH_DEPTH];
                    self.hh_score = [[[0; 64]; 64]; 2];
                    self.bf_score = [[[1; 64]; 64]; 2];
                    self.history_score = [[[0; 64]; 64]; 2];
                    self.search_statistics = SearchStatistics::default();
                    self.tc = tc;
                    self.self_stop = false;
                    self.search(max_depth, state);
                    self.tx.send(()).expect("Error sending finish flag!");
                }
            }
        }
    }

    fn search(&mut self, max_depth: i16, state: GameState) {
        if self.itcs.uci_options().debug_print {
            println!(
                "info String Thread {} starting the search of state!",
                self.id
            );
        }
        let mut curr_depth = 0;
        let mut previous_score: Option<i16> = None;
        loop {
            let temp = self.itcs.get_next_depth(curr_depth);
            curr_depth = temp.0;
            self.main_thread_in_depth = temp.1;
            if curr_depth as i16 > max_depth {
                break;
            }
            //Start Aspiration Window
            if self.itcs.uci_options().debug_print {
                println!(
                    "info String Thread {} starting aspiration window with depth {}",
                    self.id, curr_depth
                );
            }
            let mut delta = if let Some(ps) = previous_score {
                ps.abs() / 50
            } else {
                0
            } + 14;
            let mut alpha = if curr_depth == 1 {
                -16000
            } else {
                self.current_pv.score - delta
            };
            let mut beta = if curr_depth == 1 {
                16000
            } else {
                self.current_pv.score + delta
            };
            loop {
                principal_variation_search(
                    CombinedSearchParameters::from(
                        alpha,
                        beta,
                        curr_depth as i16,
                        &state,
                        if state.get_color_to_move() == WHITE {
                            1
                        } else {
                            -1
                        },
                        0,
                    ),
                    self,
                );
                if self.self_stop {
                    break;
                }
                if self.current_pv.score > alpha && self.current_pv.score < beta {
                    break;
                }

                if self.current_pv.score <= alpha {
                    if alpha < -10000 || self.current_pv.score < MATED_IN_MAX {
                        alpha = -16000;
                        beta = 16000;
                    } else {
                        beta = (alpha + beta) / 2;
                        alpha -= delta;
                    }
                }
                if self.current_pv.score >= beta {
                    if beta > 10000 || self.current_pv.score > -MATED_IN_MAX {
                        beta = 16000;
                        alpha = -16000;
                    } else {
                        beta += delta;
                    }
                }
                delta = (f64::from(delta) * 1.5) as i16;
            }
            previous_score = Some(self.current_pv.score);
            if self.self_stop {
                break;
            }
        }
        if self.itcs.uci_options().debug_print {
            println!(
                "info String Thread {} stopping the search of state!",
                self.id
            );
        }
        //Report nodes in the end
        self.itcs.update(
            self.id,
            self.search_statistics.nodes_searched,
            self.search_statistics.seldepth,
        );
        if self.id == 0 {
            *self
                .itcs
                .timeout_flag
                .write()
                .expect("Couldn't write to timeout flag") = true;
        }
    }
}

pub fn search_move(
    itcs: Arc<InterThreadCommunicationSystem>,
    max_depth: i16,
    game_state: GameState,
    history: Vec<GameState>,
    tc: TimeControl,
) -> Option<i16> {
    //1. Prepare itcs (reset things from previous search)
    *itcs.best_pv.lock().unwrap() = ScoredPrincipalVariation::default();
    itcs.stable_pv.store(false, Ordering::Relaxed);
    *itcs.depth_info.lock().unwrap() = [DepthInformation::UnSearched; MAX_SEARCH_DEPTH];
    itcs.nodes_searched()
        .iter()
        .for_each(|x| x.store(0u64, Ordering::Relaxed));
    itcs.seldepth.store(0, Ordering::Relaxed);
    *itcs.start_time.write().unwrap() = Instant::now();
    *itcs.last_cache_status.lock().unwrap() = None;
    itcs.cache_status.store(0, Ordering::Relaxed);
    itcs.cache().increase_age();
    *itcs.timeout_flag.write().unwrap() = false;

    let time_saved_before = itcs.saved_time.load(Ordering::Relaxed);
    //Step 1. Check how many legal moves there are
    let mut movelist = MoveList::default();
    generate_moves(&game_state, false, &mut movelist);

    //Step2. Check legal moves
    if movelist.move_list.is_empty() {
        panic!("The root position given does not have any legal move!");
    } else if movelist.move_list.len() == 1 {
        println!("bestmove {:?}", movelist.move_list[0].0);

        let new_timesaved: u64 = (time_saved_before as i64
            + tc.time_saved(0, time_saved_before, itcs.uci_options().move_overhead))
        .max(0) as u64;
        itcs.saved_time.store(new_timesaved, Ordering::Relaxed);
        return None;
    }

    //Step3. Prepare history
    let mut hist: History = History::default();
    let mut relevant_hashes: Vec<u64> = Vec::with_capacity(100);
    for gs in history.iter().rev() {
        relevant_hashes.push(gs.get_hash());
        if gs.get_half_moves() == 0 {
            break;
        }
    }
    for hashes in relevant_hashes.iter().rev() {
        hist.push(*hashes, false);
    }

    //Step 4. Send search command
    for tx in itcs.tx.read().unwrap().iter() {
        tx.send(ThreadInstruction::StartSearch(
            max_depth,
            game_state.clone(),
            tc,
            hist.clone(),
            time_saved_before,
        ))
        .expect("Couldn't send search command!");
    }

    //Step 5. Wait until every thread finished up
    for _ in 0..itcs.uci_options().threads {
        itcs.rx_f
            .recv()
            .expect("Could not receive finish flag from channel");
    }

    //Step 6. Report to UCI
    itcs.report_bestmove();
    //Store new saved time
    let elapsed_time = itcs.get_time_elapsed();
    let new_timesaved: u64 = (time_saved_before as i64
        + tc.time_saved(
            elapsed_time,
            time_saved_before,
            itcs.uci_options().move_overhead,
        ))
    .max(0) as u64;
    itcs.saved_time.store(new_timesaved, Ordering::Relaxed);
    //And return
    let best_score = itcs.best_pv.lock().unwrap().score;
    Some(best_score)
}
