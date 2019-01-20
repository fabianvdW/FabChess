use super::{Evaluation, ParallelEvaluation};

pub struct PSQT {
    pawns: u64,
    knights: u64,
    bishops: u64,
    rooks: u64,
    queens: u64,
    king: u64,
}

impl PSQT {
    pub fn new(pawns: u64, knights: u64, bishops: u64, rooks: u64, queens: u64, king: u64) -> PSQT {
        PSQT {
            pawns,
            knights,
            bishops,
            rooks,
            queens,
            king,
        }
    }
    pub fn copy(&self) -> PSQT {
        PSQT::new(self.pawns, self.knights, self.bishops, self.rooks, self.queens, self.king)
    }
}

impl Evaluation for PSQT {
    fn eval_mg(&self) -> f64 {
        0.0
    }
    fn eval_eg(&self) -> f64 {
        0.0
    }
}

impl ParallelEvaluation for PSQT {
    fn eval_mg_eg(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
}