pub mod card;
pub mod config;
pub mod scheduler;

pub const INITIAL_EASE_FACTOR: i32 = 2_500; // 250%

#[derive(PartialEq, Clone)]
pub enum CardType {
    New = 0,
    Learn = 1,
    Review = 2,
    Relearn = 3,
}

#[derive(PartialEq, Clone)]
pub enum CardQueue {
    New = 0,
    // due is the order cards are shown in
    Learn = 1,
    // due is a unix timestamp
    Review = 2,
    // due is days since epoch
    DayLearn = 3, // due is days since epoch

    /// cards are not due in these states
    Suspended = -1,
    Buried = -2,
}

#[derive(Clone)]
pub struct Card {
    pub(crate) card_type: CardType,
    pub(crate) card_queue: CardQueue,
    pub(crate) due: i64,
    pub(crate) interval: i32,
    pub(crate) ease_factor: i32,
    pub(crate) reps: i32,
    pub(crate) lapses: i32,
    pub(crate) remaining_steps: i32,
}

#[derive(Clone)]
pub enum NewCardOrder {
    Due = 0,
    Random = 1,
}

#[derive(Clone)]
pub struct Config {
    pub learn_steps: Vec<f32>,
    pub relearn_steps: Vec<f32>,

    pub cap_answer_time: i32,
    pub visible_time: i32,

    pub new_per_day: i32,
    pub reviews_per_day: i32,

    pub bury_new: bool,
    pub bury_reviews: bool,

    pub initial_ease: f32,

    pub easy_multiplier: f32,
    pub hard_multiplier: f32,
    pub lapse_multiplier: f32,
    pub interval_multiplier: f32,

    pub maximum_review_interval: i32,
    pub minimum_review_interval: i32,

    pub graduating_interval_good: i32,
    pub graduating_interval_easy: i32,

    pub new_card_order: NewCardOrder,
    pub leech_threshold: i32,
}

#[derive(Clone, Copy)]
pub enum Choice {
    Again = 1,
    Hard = 2,
    Ok = 3,
    Easy = 4,
}

trait Sched {
    fn answer_card(&mut self, choice: Choice);

    fn bury_card(&mut self);
    fn unbury_card(&mut self);

    fn suspend_card(&mut self);
    fn unsuspend_card(&mut self);

    fn schedule_card_as_new(&mut self);
    fn schedule_card_as_review(&mut self, min_days: i32, max_days: i32);
}

pub struct Scheduler {
    card: Card,
    config: Config,
    day_cut_off: i64,
    day_today: i64,
}
