const MOVE_OVERHEAD: u64 = 25;

pub struct TimeControlInformation {
    pub time_saved: u64,
    pub stable_pv: bool,
    pub high_score_diff: bool,
}

impl TimeControlInformation {
    pub fn new(time_saved: u64) -> Self {
        TimeControlInformation {
            time_saved,
            stable_pv: false,
            high_score_diff: false,
        }
    }
}

#[derive(Clone)]
pub enum TimeControl {
    Incremental(u64, u64),
    MoveTime(u64),
    Infinite,
    Tournament(u64, u64, usize),
}

impl TimeControl {
    pub fn time_over(&self, time_spent: u64, tc_information: &TimeControlInformation) -> bool {
        if let TimeControl::Incremental(mytime, myinc) = self {
            if time_spent as isize > *mytime as isize - 4 * MOVE_OVERHEAD as isize {
                return true;
            }
            let normal_time = ((*mytime as f64 - tc_information.time_saved as f64) / 30.0) as u64
                + myinc
                - MOVE_OVERHEAD;
            let time_aspired = if tc_information.time_saved < normal_time {
                ((normal_time as f64 * 0.85) as u64).max(*myinc)
            } else {
                normal_time.max(*myinc)
            };
            if time_spent < time_aspired {
                return false;
            }
            if tc_information.stable_pv && !tc_information.high_score_diff {
                return true;
            }
            if tc_information.high_score_diff {
                return time_spent as f64 > 0.85 * (normal_time + tc_information.time_saved) as f64;
            }
            //Non stable pv so we increase time
            return time_spent as f64 > 1.15 * (normal_time + tc_information.time_saved) as f64;
        } else if let TimeControl::MoveTime(move_time) = self {
            return time_spent > move_time - MOVE_OVERHEAD || *move_time < MOVE_OVERHEAD;
        } else if let TimeControl::Infinite = self {
            return false;
        } else if let TimeControl::Tournament(mytime, myinc, movestogo) = self {
            if time_spent as isize > *mytime as isize - 4 * MOVE_OVERHEAD as isize {
                return true;
            }
            let normal_time = ((*mytime as f64 - tc_information.time_saved as f64)
                / *movestogo as f64) as u64
                + myinc
                - MOVE_OVERHEAD;
            let time_aspired = if tc_information.time_saved < normal_time {
                (normal_time as f64 * 0.85) as u64
            } else {
                normal_time
            };
            if time_spent < time_aspired {
                return false;
            }
            if tc_information.stable_pv {
                return true;
            }
            //Non stable pv so we increase time
            return time_spent as f64 > 1.15 * (normal_time + tc_information.time_saved) as f64;
        }
        panic!("Invalid Timecontrol");
    }

    pub fn time_saved(&self, time_spent: u64, saved: u64) -> i64 {
        if let TimeControl::Incremental(mytime, myinc) = self {
            let normal_timecontrol =
                ((*mytime as f64 - saved as f64) / 30.0) as u64 + myinc - MOVE_OVERHEAD;
            normal_timecontrol as i64 - time_spent as i64
        } else if let TimeControl::Tournament(mytime, myinc, movestogo) = self {
            let normal_timecontrol = ((*mytime as f64 - saved as f64) / *movestogo as f64) as u64
                + myinc
                - MOVE_OVERHEAD;
            normal_timecontrol as i64 - time_spent as i64
        } else {
            0
        }
    }

    pub fn to_string(&self, tc_information: &TimeControlInformation) -> String {
        let mut res_str: String = String::new();
        if let TimeControl::Incremental(mytime, myinc) = self {
            res_str.push_str(&format!("My Time: {}\n", mytime));
            res_str.push_str(&format!("My Inc: {}\n", myinc));
            let normal_time = ((*mytime as f64 - tc_information.time_saved as f64) / 30.0) as u64
                + myinc
                - MOVE_OVERHEAD;
            let time_aspired = if tc_information.time_saved < normal_time {
                ((normal_time as f64 * 0.85) as u64).max(*myinc)
            } else {
                normal_time.max(*myinc)
            };
            res_str.push_str(&format!("My normal time I would spend: {}\n", normal_time));
            res_str.push_str(&format!(
                "My aspired time I would spend: {}\n",
                time_aspired
            ));
        } else if let TimeControl::MoveTime(time) = self {
            res_str.push_str(&format!("Limited movetime: {}\n", time));
        } else if let TimeControl::Infinite = self {
            res_str.push_str("Infinite Time!\n");
        } else if let TimeControl::Tournament(mytime, myinc, movestogo) = self {
            res_str.push_str(&format!("My Time: {}\n", mytime));
            res_str.push_str(&format!("My Inc: {}\n", myinc));
            res_str.push_str(&format!("Moves to go : {}\n", movestogo));
            let normal_time = ((*mytime as f64 - tc_information.time_saved as f64)
                / *movestogo as f64) as u64
                + myinc
                - MOVE_OVERHEAD;
            let time_aspired = if tc_information.time_saved < normal_time {
                (normal_time as f64 * 0.85) as u64
            } else {
                normal_time
            };
            res_str.push_str(&format!("My normal time I would spend: {}\n", normal_time));
            res_str.push_str(&format!(
                "My aspired time I would spend: {}\n",
                time_aspired
            ));
        }

        res_str
    }
}
