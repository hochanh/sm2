use crate::srs::config::INITIAL_EASE_FACTOR;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_new_position() {
        let mut card = Card::default();
        card.set_new_position(1);
        assert_eq!(card.due, 1);

        card.card_queue = CardQueue::Review;
        card.card_type = CardType::Review;
        card.set_new_position(2);
        assert_eq!(card.due, 1);
    }
}
