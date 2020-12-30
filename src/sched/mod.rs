use crate::card::Card;
use crate::conf::Config;

mod scheduler;

#[derive(Clone, Copy)]
pub enum Choice {
    Again = 1,
    Hard = 2,
    Ok = 3,
    Easy = 4,
}

trait Sched {
    fn answer_card(&mut self, choice: Choice);
    fn reset_card(&mut self);
}

pub struct Scheduler {
    card: Card,
    config: Config,
    day_cut_off: i64,
    day_today: i64,
}
