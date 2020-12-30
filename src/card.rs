use crate::conf::INITIAL_EASE_FACTOR;
use crate::service::time::Timestamp;

#[derive(PartialEq)]
pub enum CardType {
    New = 0,
    Learn = 1,
    Review = 2,
    Relearn = 3,
}

#[derive(PartialEq)]
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
    SchedBuried = -2,
    UserBuried = -3,
}

pub struct Card {
    pub(crate) id: i64,
    pub(crate) card_type: CardType,
    pub(crate) card_queue: CardQueue,
    pub(crate) due: i64,
    pub(crate) interval: i32,
    pub(crate) ease_factor: i32,
    pub(crate) reps: i32,
    pub(crate) lapses: i32,
    pub(crate) remaining_steps: i32,
    pub(crate) modified_at: i64,
    pub(crate) inserted_at: i64,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            id: 0,
            card_type: CardType::New,
            card_queue: CardQueue::New,
            due: 0,
            interval: 0,
            ease_factor: 0,
            reps: 0,
            lapses: 0,
            remaining_steps: 0,
            modified_at: Timestamp::now(),
            inserted_at: Timestamp::now(),
        }
    }
}

impl Card {
    fn new(due: i64) -> Self {
        let mut card = Card::default();
        card.due = due;
        card
    }

    fn set_modified_at(&mut self, modified_at: i64) {
        self.modified_at = modified_at
    }

    fn schedule_as_new(&mut self, position: i64) {
        self.due = position as i64;
        self.card_type = CardType::New;
        self.card_queue = CardQueue::New;
        self.interval = 0;
        self.ease_factor = INITIAL_EASE_FACTOR;
    }

    fn set_new_position(&mut self, position: i64) {
        if self.card_queue != CardQueue::New || self.card_type != CardType::New {
            return;
        }
        self.due = position;
    }
}
