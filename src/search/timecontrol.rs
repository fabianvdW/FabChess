const MOVE_OVERHEAD: u64 = 20;

pub struct TimeControlInformation {
    pub time_saved: u64,
    pub stable_pv: bool,
}
impl TimeControlInformation {
    pub fn new(time_saved: u64) -> Self {
        TimeControlInformation {
            time_saved,
            stable_pv: false,
        }
    }
}
#[derive(Clone)]
pub enum TimeControl {
    Incremental(u64, u64),
    MoveTime(u64),
    Infinite,
}

impl TimeControl {
    pub fn time_over(&self, time_spent: u64, tc_information: &TimeControlInformation) -> bool {
        if let TimeControl::Incremental(mytime, myinc) = self {
            if time_spent > mytime - MOVE_OVERHEAD {
                return true;
            }
            let normal_time = (*mytime as f64 / 30.0) as u64 + myinc - MOVE_OVERHEAD;
            let time_aspired = (normal_time as f64 * 0.85) as u64;
            if time_spent < time_aspired {
                return false;
            }
            if tc_information.stable_pv {
                return true;
            }
            //Non stable pv so we increase time
            return time_spent as f64 > 1.15 * (normal_time + tc_information.time_saved) as f64;
        } else if let TimeControl::MoveTime(move_time) = self {
            return time_spent > move_time - MOVE_OVERHEAD || *move_time < MOVE_OVERHEAD;
        } else if let TimeControl::Infinite = self {
            return false;
        }
        panic!("Invalid Timecontrol");
    }

    pub fn time_saved(&self, time_spent: u64) -> i64 {
        if let TimeControl::Incremental(mytime, myinc) = self {
            let normal_timecontrol = (*mytime as f64 / 30.0) as u64 + myinc - MOVE_OVERHEAD;
            return normal_timecontrol as i64 - time_spent as i64;
        } else {
            0
        }
    }
}
