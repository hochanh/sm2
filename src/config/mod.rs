mod config;

pub const INITIAL_EASE_FACTOR: u16 = 2_500; // 250%

pub enum NewCardOrder {
    Due = 0,
    Random = 1,
}

pub enum LeechAction {
    Suspend = 0,
    Tag = 1,
}

pub struct InnerConfig {
    learn_steps: Vec<f32>,
    relearn_steps: Vec<f32>,

    cap_answer_time: i32,
    visible_time: i32,

    new_per_day: i32,
    reviews_per_day: i32,

    bury_new: bool,
    bury_reviews: bool,

    initial_ease: f32,

    easy_multiplier: f32,
    hard_multiplier: f32,
    lapse_multiplier: f32,
    interval_multiplier: f32,

    maximum_review_interval: i32,
    minimum_review_interval: i32,

    graduating_interval_good: i32,
    graduating_interval_easy: i32,

    new_card_order: NewCardOrder,
    leech_action: LeechAction,
    leech_threshold: i32,
}

pub struct Config {
    pub id: i64,
    pub name: String,
    pub inner: InnerConfig,
    pub modified_at: i64,
    pub inserted_at: i64,
}