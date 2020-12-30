mod config;

pub const INITIAL_EASE_FACTOR: i32 = 2_500; // 250%

pub enum NewCardOrder {
    Due = 0,
    Random = 1,
}

pub enum LeechAction {
    Suspend = 0,
    Tag = 1,
}

pub struct InnerConfig {
    pub learn_steps: Vec<u32>,
    pub relearn_steps: Vec<u32>,

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
    pub leech_action: LeechAction,
    pub leech_threshold: i32,
}

pub struct Config {
    pub id: i64,
    pub name: String,
    pub inner: InnerConfig,
    pub modified_at: i64,
    pub inserted_at: i64,
}