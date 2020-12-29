use crate::scheduler::{Scheduler, Choice};
use crate::card::{Card, CardQueue, CardType};
use crate::config::Config;
use crate::service::time::Timestamp;

pub struct Learn {
    card: Card,
    config: Config,
}

impl Scheduler for Learn {
    fn answer_card(&mut self, choice: Choice) {
        self.card.card_queue = CardQueue::Learn;
        self.card.card_type = CardType::Learn;
        self.card.due = Timestamp::now() + 1_000
    }

    fn next_interval(&self, choice: Choice) -> u32 {
        unimplemented!()
    }

    fn bury_card(&mut self) {
        unimplemented!()
    }

    fn unbury_card(&mut self) {
        unimplemented!()
    }

    fn schedule_as_new(&mut self) {
        unimplemented!()
    }

    fn schedule_as_review(&mut self, min_interval: u32, max_interval: u32) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardType;
    use crate::service::time::Timestamp;

    #[test]
    fn test_new() {
        let mut learn = Learn { card: Card::default(), config: Config::default()};
        learn.answer_card(Choice::Again);
        assert!(matches!(learn.card.card_queue, CardQueue::Learn));
        assert!(matches!(learn.card.card_type, CardType::Learn));
        assert!(learn.card.due > Timestamp::now());
    }
}