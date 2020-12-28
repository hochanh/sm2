use crate::card::{Card, CardType, CardQueue};
use crate::service::time::Timestamp;

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
    pub fn new(due: i32) -> Self {
        let mut card = Card::default();
        card.due = due;
        card
    }

    pub fn set_modified(&mut self, modified_at: i64) {
        self.modified_at = modified_at
    }
}