use std::cmp::{max, min};

use rand::Rng;

use crate::service::time::Timestamp;
use crate::srs::{
    Card, CardQueue, CardType, Choice, Config, Sched, Scheduler, INITIAL_EASE_FACTOR,
};

impl Scheduler {
    pub fn new(card: Card, config: Config, day_cut_off: i64) -> Self {
        Self {
            card,
            config,
            day_cut_off,
            day_today: day_cut_off / 86_400,
        }
    }
}

impl Sched for Scheduler {
    fn answer_card(&mut self, choice: Choice) {
        self.answer(choice);
    }

    fn reset_card(&mut self) {
        unimplemented!()
    }
}

impl Scheduler {
    fn answer(&mut self, choice: Choice) {
        self.card.reps += 1;

        if matches!(self.card.card_queue, CardQueue::New) {
            self.card.card_queue = CardQueue::Learn;
            self.card.card_type = CardType::Learn;
            self.card.remaining_steps = self.start_remaining_steps();
        }

        match self.card.card_queue {
            CardQueue::Learn | CardQueue::DayLearn => {
                self.answer_learn_card(choice);
            }
            CardQueue::Review => {
                self.answer_review_card(choice);
            }
            _ => {}
        }
    }

    fn start_remaining_steps(&mut self) -> i32 {
        let steps = match self.card.card_type {
            CardType::Relearn => &self.config.relearn_steps,
            _ => &self.config.learn_steps,
        };

        let total_steps = steps.len();
        let total_remaining = self.remaining_today(steps, total_steps);
        total_steps as i32 + total_remaining * 1_000
    }

    // The number of steps that can be completed by the day cutoff
    fn remaining_today(&self, steps: &[f32], remaining: usize) -> i32 {
        let mut now = Timestamp::now();
        let remaining_steps = &steps[steps.len() - remaining..steps.len()];
        let mut remain = 0;
        for (i, item) in remaining_steps.iter().enumerate() {
            now += (item * 60.0) as i64;
            if now > self.day_cut_off {
                break;
            }
            remain = i
        }
        (remain + 1) as i32
    }

    fn answer_learn_card(&mut self, choice: Choice) {
        match choice {
            Choice::Easy => self.reschedule_as_review(true),
            Choice::Ok => {
                if self.card.remaining_steps % 1_000 <= 1 {
                    self.reschedule_as_review(false)
                } else {
                    self.move_to_next_step()
                }
            }
            Choice::Hard => self.repeat_step(),
            Choice::Again => self.move_to_first_step(),
        }
    }

    fn reschedule_as_review(&mut self, early: bool) {
        match self.card.card_type {
            CardType::Review | CardType::Relearn => self.reschedule_graduating_lapse(early),
            _ => self.reschedule_new(early),
        }
    }

    fn reschedule_graduating_lapse(&mut self, early: bool) {
        if early {
            self.card.interval += 1
        }
        self.card.due = self.day_today + self.card.interval as i64;
        self.card.card_type = CardType::Review;
        self.card.card_queue = CardQueue::Review;
    }

    fn reschedule_new(&mut self, early: bool) {
        self.card.interval = self.graduating_interval(early, true);
        self.card.due = self.day_today + self.card.interval as i64;
        self.card.ease_factor = INITIAL_EASE_FACTOR;
        self.card.card_queue = CardQueue::Review;
        self.card.card_type = CardType::Review;
    }

    fn graduating_interval(&mut self, early: bool, fuzzy: bool) -> i32 {
        match self.card.card_type {
            CardType::Review | CardType::Relearn => {
                let bonus = if early { 1 } else { 0 };
                self.card.interval + bonus
            }
            _ => {
                let ideal = if early {
                    self.config.graduating_interval_easy
                } else {
                    self.config.graduating_interval_good
                };

                if fuzzy {
                    Scheduler::fuzz_interval(ideal)
                } else {
                    ideal
                }
            }
        }
    }

    fn fuzz_interval(interval: i32) -> i32 {
        let (min, max) = Scheduler::fuzz_interval_range(interval);
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }

    fn fuzz_interval_range(interval: i32) -> (i32, i32) {
        match interval {
            0..=1 => (1, 1),
            2 => (2, 3),
            _ => {
                let fuzz = if interval < 7 {
                    (interval as f32 * 0.25) as i32
                } else if interval < 30 {
                    max(2, (interval as f32 * 0.15) as i32)
                } else {
                    max(4, (interval as f32 * 0.05) as i32)
                };
                let fuzz_int = max(fuzz, 1);
                (interval - fuzz_int, interval + fuzz_int)
            }
        }
    }

    fn answer_review_card(&mut self, choice: Choice) {
        let early = false;
        match choice {
            Choice::Again => self.reschedule_lapse(),
            _ => self.reschedule_review(choice, early),
        }
    }

    fn reschedule_review(&mut self, choice: Choice, early: bool) {
        if early {
            self.update_early_review_interval(choice)
        } else {
            self.update_review_interval(choice)
        }

        self.card.ease_factor = max(
            1_300,
            self.card.ease_factor + vec![-150, 0, 150][choice as usize - 2],
        );
        self.card.due = self.day_today + self.card.interval as i64;
    }

    fn update_early_review_interval(&mut self, choice: Choice) {
        self.card.interval = self.early_review_interval(choice)
    }

    fn early_review_interval(&self, choice: Choice) -> i32 {
        let elapsed = self.day_today + self.card.interval as i64;

        let mut easy_bonus = 1.0;
        let mut min_new_interval = 1;
        let factor: f32;

        match choice {
            Choice::Hard => {
                factor = self.config.hard_multiplier;
                min_new_interval = (factor / 2.0) as i32;
            }
            Choice::Ok => {
                factor = self.card.ease_factor as f32 / 1_000.0;
            }
            _ => {
                factor = self.card.ease_factor as f32 / 1_000.0;
                let bonus = self.config.easy_multiplier;
                easy_bonus = bonus - (bonus - 1.0) / 2.0
            }
        }

        let mut interval = f32::max(elapsed as f32 * factor, 1.0);
        interval = f32::max((self.card.interval * min_new_interval) as f32, interval) * easy_bonus;
        self.constrain_interval(interval, 0, false)
    }

    fn constrain_interval(&self, interval: f32, previous: i32, fuzzy: bool) -> i32 {
        let mut interval = (interval * self.config.interval_multiplier) as i32;
        if fuzzy {
            interval = Scheduler::fuzz_interval(interval);
        }
        interval = max(max(interval as i32, previous + 1), 1);
        min(interval, self.config.maximum_review_interval)
    }

    fn update_review_interval(&mut self, choice: Choice) {
        self.card.interval = self.next_review_interval(choice, true)
    }

    fn next_review_interval(&self, choice: Choice, fuzzy: bool) -> i32 {
        let factor = self.card.ease_factor / 1_000;
        let delay = self.days_late();
        let hard_factor = self.config.hard_multiplier;
        let hard_min = if hard_factor > 1.0 {
            self.card.interval
        } else {
            0
        } as i32;
        let mut interval =
            self.constrain_interval(self.card.interval as f32 * hard_factor, hard_min, fuzzy);
        if matches!(choice, Choice::Hard) {
            return interval;
        }

        interval = self.constrain_interval(
            (self.card.interval as f32 + delay as f32 / 2.0) * factor as f32,
            interval,
            fuzzy,
        );
        if matches!(choice, Choice::Ok) {
            return interval;
        }

        self.constrain_interval(
            ((self.card.interval + delay) * factor) as f32 * self.config.easy_multiplier,
            interval,
            fuzzy,
        )
    }

    fn reschedule_lapse(&mut self) {
        self.card.lapses += 1;
        self.card.ease_factor = max(1_300, self.card.ease_factor - 200);

        let suspended = self.check_leech() && matches!(self.card.card_queue, CardQueue::Suspended);

        if !self.config.relearn_steps.is_empty() && !suspended {
            self.card.card_type = CardType::Relearn;
            self.move_to_first_step();
        } else {
            self.update_review_interval_on_fail();
            self.reschedule_as_review(false);

            if suspended {
                self.card.card_queue = CardQueue::Suspended;
            }
        }
    }

    fn update_review_interval_on_fail(&mut self) {
        self.card.interval = self.lapse_interval();
    }

    fn lapse_interval(&self) -> i32 {
        max(
            1,
            max(
                self.config.minimum_review_interval,
                (self.card.interval as f32 * self.config.lapse_multiplier) as i32,
            ),
        )
    }

    fn check_leech(&self) -> bool {
        let lt = self.config.leech_threshold;
        if lt == 0 {
            false
        } else {
            self.card.lapses >= lt && (self.card.lapses - lt) % (max(lt / 2, 1)) == 0
        }
    }

    fn days_late(&self) -> i32 {
        max(0, self.day_today - self.card.due) as i32
    }

    fn move_to_next_step(&mut self) {
        let remaining = (self.card.remaining_steps % 1_000) - 1;
        self.card.remaining_steps =
            self.remaining_today(&self.config.learn_steps, remaining as usize) * 1_000 + remaining;

        self.reschedule_learn_card(None);
    }

    fn repeat_step(&mut self) {
        let delay = self.delay_for_repeating_grade(self.card.remaining_steps);
        self.reschedule_learn_card(Some(delay))
    }

    fn reschedule_learn_card(&mut self, delay: Option<i32>) {
        let delay = match delay {
            None => self.delay_for_repeating_grade(self.card.remaining_steps),
            Some(value) => value,
        };

        self.card.due = Timestamp::now() + delay as i64;

        if self.card.due < self.day_cut_off {
            let max_extra = min(300, (delay as f32 * 0.25) as i64);
            let mut rng = rand::thread_rng();
            let fuzz = rng.gen_range(0..=max(1, max_extra));
            self.card.due = min(self.day_cut_off - 1, self.card.due + fuzz);
            self.card.card_queue = CardQueue::Learn;
        } else {
            let ahead = ((self.card.due - self.day_cut_off) / 86_400) + 1;
            self.card.due = self.day_today + ahead;
            self.card.card_queue = CardQueue::DayLearn;
        }
    }

    fn delay_for_repeating_grade(&self, remaining: i32) -> i32 {
        let delay1 = self.delay_for_grade(remaining);
        let delay2 = if !self.config.relearn_steps.is_empty() {
            self.delay_for_grade(remaining)
        } else {
            delay1 * 2
        };
        (delay1 + max(delay1, delay2)) / 2
    }

    fn delay_for_grade(&self, remaining: i32) -> i32 {
        let left = remaining % 1_000;
        let index = self.config.learn_steps.len() - left as usize;
        let delay = self.config.learn_steps[index];
        (delay * 60.0) as i32
    }

    fn move_to_first_step(&mut self) {
        self.card.remaining_steps = self.start_remaining_steps();
        if matches!(self.card.card_type, CardType::Relearn) {
            self.update_review_interval_on_fail()
        }
        self.reschedule_learn_card(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::service::time::Timestamp;
    use crate::srs::CardType;

    use super::*;

    #[test]
    fn test_new() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());
        scheduler.answer_card(Choice::Again);
        assert!(matches!(scheduler.card.card_queue, CardQueue::Learn));
        assert!(matches!(scheduler.card.card_type, CardType::Learn));
        assert!(scheduler.card.due >= Timestamp::now());
    }

    #[test]
    fn test_learn() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());

        // Fail it
        scheduler.config.learn_steps = vec![0.5, 3.0, 10.0];
        scheduler.answer(Choice::Again);
        // Got 3 steps before graduation
        assert_eq!(scheduler.card.remaining_steps % 1_000, 3);
        assert_eq!(scheduler.card.remaining_steps / 1_000, 3);
        // Due in 30 seconds
        let t1 = scheduler.card.due - Timestamp::now();
        assert!(t1 >= 25 && t1 <= 40);

        // Pass it once
        scheduler.answer(Choice::Ok);
        // Due in 3 minutes
        let t2 = scheduler.card.due - Timestamp::now();
        assert!(t2 >= 178 && t2 <= 225);
        assert_eq!(scheduler.card.remaining_steps % 1_000, 2);
        assert_eq!(scheduler.card.remaining_steps / 1_000, 2);

        // Pass again
        scheduler.answer(Choice::Ok);
        // Due in 10 minutes
        let t3 = scheduler.card.due - Timestamp::now();
        assert!(t3 >= 599 && t3 <= 750);
        assert_eq!(scheduler.card.remaining_steps % 1_000, 1);
        assert_eq!(scheduler.card.remaining_steps / 1_000, 1);

        // Graduate the card
        assert!(matches!(scheduler.card.card_type, CardType::Learn));
        assert!(matches!(scheduler.card.card_queue, CardQueue::Learn));
        scheduler.answer(Choice::Ok);
        assert!(matches!(scheduler.card.card_type, CardType::Review));
        assert!(matches!(scheduler.card.card_queue, CardQueue::Review));
        // Due tomorrow with interval of 1
        assert_eq!(scheduler.card.due, scheduler.day_today + 1);
        assert_eq!(scheduler.card.interval, 1);
        // Or normal removal
        scheduler.card.card_type = CardType::New;
        scheduler.card.card_queue = CardQueue::Learn;
        scheduler.answer(Choice::Easy);
        assert!(matches!(scheduler.card.card_type, CardType::Review));
        assert!(matches!(scheduler.card.card_queue, CardQueue::Review));
        let (min, max) = Scheduler::fuzz_interval_range(4);
        assert!(scheduler.card.interval >= min && scheduler.card.interval <= max);
    }

    #[test]
    fn test_relearn() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());
        scheduler.card.interval = 100;
        scheduler.card.due = scheduler.day_today;
        scheduler.card.card_queue = CardQueue::Review;
        scheduler.card.card_type = CardType::Review;

        // Fail the card
        scheduler.answer(Choice::Again);
        assert!(matches!(scheduler.card.card_type, CardType::Relearn));
        assert!(matches!(scheduler.card.card_queue, CardQueue::Learn));
        assert_eq!(scheduler.card.interval, 1);

        // Immediately graduate it
        scheduler.answer(Choice::Easy);
        assert!(matches!(scheduler.card.card_type, CardType::Review));
        assert!(matches!(scheduler.card.card_queue, CardQueue::Review));
        assert_eq!(scheduler.card.interval, 2);
        assert_eq!(
            scheduler.card.due,
            scheduler.day_today + scheduler.card.interval as i64
        );
    }

    #[test]
    fn test_relearn_no_steps() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());
        scheduler.card.interval = 100;
        scheduler.card.due = scheduler.day_today;
        scheduler.card.card_queue = CardQueue::Review;
        scheduler.card.card_type = CardType::Review;

        scheduler.config.relearn_steps = vec![];
        // Fail the card
        scheduler.answer(Choice::Again);
        assert!(matches!(scheduler.card.card_type, CardType::Review));
        assert!(matches!(scheduler.card.card_queue, CardQueue::Review));
    }
}
