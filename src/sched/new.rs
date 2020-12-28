use crate::card::{Card, CardType, CardQueue};
use crate::config::INITIAL_EASE_FACTOR;

impl Card {
    fn schedule_as_new(&mut self, position: u32) {
        self.due = position as i32;
        self.card_type = CardType::New;
        self.card_queue = CardQueue::New;
        self.interval = 0;
        self.ease_factor = INITIAL_EASE_FACTOR;
    }

    /// If the card is new, change its position.
    fn set_new_position(&mut self, position: u32) {
        if self.card_queue != CardQueue::New || self.card_type != CardType::New {
            return;
        }
        self.due = position as i32;
    }
}