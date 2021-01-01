use crate::srs::{Config, LeechAction, NewCardOrder};

impl Default for Config {
    fn default() -> Self {
        Self {
            learn_steps: vec![1.0, 10.0],
            relearn_steps: vec![10.0],
            cap_answer_time: 60,
            visible_time: 0,
            new_per_day: 20,
            reviews_per_day: 200,
            bury_new: false,
            bury_reviews: false,
            initial_ease: 2.5,
            easy_multiplier: 1.3,
            hard_multiplier: 1.2,
            lapse_multiplier: 0.0,
            interval_multiplier: 1.0,
            maximum_review_interval: 36_500,
            minimum_review_interval: 1,
            graduating_interval_good: 1,
            graduating_interval_easy: 4,
            new_card_order: NewCardOrder::Due,
            leech_action: LeechAction::Suspend,
            leech_threshold: 8,
        }
    }
}
