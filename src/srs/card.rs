use crate::service::time::Timestamp;
use crate::srs::{Card, CardQueue, CardType, INITIAL_EASE_FACTOR};

impl Default for Card {
    fn default() -> Self {
        Self {
            card_type: CardType::New,
            card_queue: CardQueue::New,
            due: 0,
            interval: 0,
            ease_factor: 0,
            reps: 0,
            lapses: 0,
            remaining_steps: 0,
        }
    }
}

impl Card {
    fn new(due: i64) -> Self {
        let mut card = Card::default();
        card.due = due;
        card
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
