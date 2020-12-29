use crate::card::Card;
use crate::config::Config;

mod learn;

pub enum Choice {
    Again = 1,
    Hard = 2,
    Ok = 3,
    Easy = 4,
}

trait Scheduler {
    fn answer_card(&mut self, choice: Choice);
    fn next_interval(&self, choice: Choice) -> u32;

    fn bury_card(&mut self);
    fn unbury_card(&mut self);

    fn schedule_as_new(&mut self);
    fn schedule_as_review(&mut self, min_interval: u32, max_interval: u32);
}