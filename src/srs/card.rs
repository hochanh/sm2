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
    pub fn new(due: i64) -> Self {
        let mut card = Card::default();
        card.due = due;
        card
    }

    pub fn schedule_as_new(&mut self, position: i64) {
        self.due = position;
        self.card_type = CardType::New;
        self.card_queue = CardQueue::New;
        self.interval = 0;
        if self.ease_factor == 0 {
            self.ease_factor = INITIAL_EASE_FACTOR;
        }
    }

    pub fn schedule_as_review(&mut self, interval: i32, today: i64) {
        self.interval = interval.max(1);
        self.due = today + interval as i64;
        self.card_type = CardType::Review;
        self.card_queue = CardQueue::Review;
        if self.ease_factor == 0 {
            self.ease_factor = INITIAL_EASE_FACTOR;
        }
    }

    pub fn set_new_position(&mut self, position: i64) {
        if self.card_queue != CardQueue::New || self.card_type != CardType::New {
            return;
        }
        self.due = position;
    }
}
