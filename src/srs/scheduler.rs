use std::cmp::{max, min};

use rand::distributions::{Distribution, Uniform};
use rand::Rng;

use crate::service::timespan::answer_button_time;
use crate::service::timestamp::Timestamp;
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

    fn fuzz_interval(interval: i32) -> i32 {
        let (min, max) = Self::fuzz_interval_range(interval);
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
}

impl Sched for Scheduler {
    fn answer_card(&mut self, choice: Choice) {
        self.answer(choice);
    }

    fn bury_card(&mut self) {
        self.card.card_queue = CardQueue::Buried;
    }

    fn unbury_card(&mut self) {
        self.card.card_queue = CardQueue::New;
    }

    fn suspend_card(&mut self) {
        self.card.card_queue = CardQueue::Suspended;
    }

    fn unsuspend_card(&mut self) {
        self.card.card_queue = match self.card.card_type {
            CardType::Learn | CardType::Relearn => {
                if self.card.due > 1_000_000_000 {
                    CardQueue::Learn
                } else {
                    CardQueue::DayLearn
                }
            }
            CardType::New => CardQueue::New,
            CardType::Review => CardQueue::Review,
        }
    }

    fn schedule_card_as_new(&mut self) {
        self.card.schedule_as_new(0);
    }

    fn schedule_card_as_review(&mut self, min_days: i32, max_days: i32) {
        let mut rng = rand::thread_rng();
        let distribution = Uniform::from(min_days..=max_days);
        let interval = distribution.sample(&mut rng);
        self.card.schedule_as_review(interval, self.day_today);
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

    fn start_remaining_steps(&self) -> i32 {
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
        let mut now = Timestamp::now() as f32;
        let from_idx = if steps.len() > remaining {
            steps.len() - remaining
        } else {
            0
        };
        let remaining_steps = &steps[from_idx..steps.len()];
        let mut remain = 0;
        let day_cut_off = self.day_cut_off as f32;
        for (i, item) in remaining_steps.iter().enumerate() {
            now += item * 60.0;
            if now > day_cut_off {
                break;
            }
            remain = i
        }
        (remain + 1) as i32
    }

    fn answer_learn_card(&mut self, choice: Choice) {
        let steps = &self.config.learn_steps.clone();
        match choice {
            Choice::Easy => self.reschedule_as_review(true),
            Choice::Ok => {
                if self.card.remaining_steps % 1_000 <= 1 {
                    self.reschedule_as_review(false)
                } else {
                    self.move_to_next_step(steps)
                }
            }
            Choice::Hard => self.repeat_step(steps),
            Choice::Again => self.move_to_first_step(steps),
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

    fn graduating_interval(&self, early: bool, fuzzy: bool) -> i32 {
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
                    Self::fuzz_interval(ideal)
                } else {
                    ideal
                }
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
            interval = Self::fuzz_interval(interval);
        }
        interval = max(max(interval as i32, previous + 1), 1);
        min(interval, self.config.maximum_review_interval)
    }

    fn update_review_interval(&mut self, choice: Choice) {
        self.card.interval = self.next_review_interval(choice, true)
    }

    fn next_review_interval(&self, choice: Choice, fuzzy: bool) -> i32 {
        let factor = self.card.ease_factor as f32 / 1_000.0;
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
            (self.card.interval as f32 + delay as f32 / 2.0) * factor,
            interval,
            fuzzy,
        );
        if matches!(choice, Choice::Ok) {
            return interval;
        }

        self.constrain_interval(
            ((self.card.interval + delay) as f32 * factor) * self.config.easy_multiplier,
            interval,
            fuzzy,
        )
    }

    fn reschedule_lapse(&mut self) {
        self.card.lapses += 1;
        self.card.ease_factor = max(1_300, self.card.ease_factor - 200);

        let leech = self.check_leech();
        // Always suspend card on leech
        if leech {
            self.card.card_queue = CardQueue::Suspended
        }
        let suspended = matches!(self.card.card_queue, CardQueue::Suspended);

        let steps = &self.config.relearn_steps.clone();
        if !steps.is_empty() && !suspended {
            self.card.card_type = CardType::Relearn;
            self.move_to_first_step(steps);
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

    fn move_to_next_step(&mut self, steps: &[f32]) {
        let remaining = (self.card.remaining_steps % 1_000) - 1;
        self.card.remaining_steps =
            self.remaining_today(steps, remaining as usize) * 1_000 + remaining;

        self.reschedule_learn_card(steps, None);
    }

    fn repeat_step(&mut self, steps: &[f32]) {
        let delay = self.delay_for_repeating_grade(steps, self.card.remaining_steps);
        self.reschedule_learn_card(steps, Some(delay))
    }

    fn reschedule_learn_card(&mut self, steps: &[f32], delay: Option<i32>) {
        let delay = match delay {
            None => self.delay_for_grade(steps, self.card.remaining_steps),
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

    fn delay_for_repeating_grade(&self, steps: &[f32], remaining: i32) -> i32 {
        let delay1 = self.delay_for_grade(steps, remaining);
        let delay2 = if steps.len() > 1 {
            self.delay_for_grade(steps, remaining - 1)
        } else {
            delay1 * 2
        };
        (delay1 + max(delay1, delay2)) / 2
    }

    fn delay_for_grade(&self, steps: &[f32], remaining: i32) -> i32 {
        let left = (remaining % 1_000) as usize;
        let delay = if steps.is_empty() {
            1.0
        } else if steps.len() >= left {
            steps[steps.len() - left]
        } else {
            steps[0]
        };
        (delay * 60.0) as i32
    }

    fn move_to_first_step(&mut self, steps: &[f32]) {
        self.card.remaining_steps = self.start_remaining_steps();
        if matches!(self.card.card_type, CardType::Relearn) {
            self.update_review_interval_on_fail()
        }
        self.reschedule_learn_card(steps, None)
    }

    fn next_interval(&self, choice: Choice) -> i32 {
        match self.card.card_queue {
            CardQueue::New | CardQueue::Learn | CardQueue::DayLearn => {
                self.next_learn_interval(choice)
            }
            _ => {
                if matches!(choice, Choice::Again) {
                    let steps = &self.config.relearn_steps;
                    if !steps.is_empty() {
                        (steps[0] * 60.0) as i32
                    } else {
                        self.lapse_interval() * 86_400
                    }
                } else {
                    self.next_review_interval(choice, false) * 86_400
                }
            }
        }
    }

    fn next_learn_interval(&self, choice: Choice) -> i32 {
        let steps = &self.config.learn_steps;
        match choice {
            Choice::Again => self.delay_for_grade(steps, steps.len() as i32),
            Choice::Hard => self.delay_for_repeating_grade(steps, steps.len() as i32),
            Choice::Easy => self.graduating_interval(true, false) * 86_400,
            Choice::Ok => {
                let remaining = if matches!(self.card.card_queue, CardQueue::New) {
                    self.start_remaining_steps()
                } else {
                    self.card.remaining_steps
                };

                let left = remaining % 1_000 - 1;

                if left <= 0 {
                    self.graduating_interval(false, false) * 86_400
                } else {
                    self.delay_for_grade(steps, left)
                }
            }
        }
    }

    fn next_interval_string(&self, choice: Choice) -> String {
        let interval_secs = self.next_interval(choice);
        answer_button_time(interval_secs as f32)
    }
}

#[cfg(test)]
mod tests {
    use crate::service::timestamp::Timestamp;
    use crate::srs::CardType;

    use super::*;

    fn check_interval(card: &Card, interval: i32) -> bool {
        let (min, max) = Scheduler::fuzz_interval_range(interval);
        card.interval >= min && card.interval <= max
    }

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
    fn test_change_steps() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());
        scheduler.config.learn_steps = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        scheduler.answer(Choice::Ok);
        scheduler.config.learn_steps = vec![1.0];
        scheduler.answer(Choice::Ok);
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
        assert!(check_interval(&scheduler.card, 4));
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

    #[test]
    fn test_learn_day() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());

        scheduler.config.learn_steps = vec![1.0, 10.0, 1440.0, 2880.0];

        // Pass it
        scheduler.answer(Choice::Ok);
        assert_eq!(scheduler.card.remaining_steps % 1_000, 3);
        assert_eq!(scheduler.card.remaining_steps / 1_000, 1);
        assert_eq!(scheduler.next_interval(Choice::Ok), 86_400);

        // Learn it
        scheduler.answer(Choice::Ok);
        assert_eq!(scheduler.card.due, scheduler.day_today + 1);
        assert!(matches!(scheduler.card.card_queue, CardQueue::DayLearn));

        // Move back a day
        scheduler.card.due -= 1;
        assert_eq!(scheduler.next_interval(Choice::Ok), 86_400 * 2);

        // Fail to answer it
        scheduler.answer(Choice::Again);
        assert!(matches!(scheduler.card.card_queue, CardQueue::Learn));

        // Ok to answer it
        scheduler.answer(Choice::Ok);
        assert_eq!(scheduler.next_interval(Choice::Ok), 86_400);
        assert!(matches!(scheduler.card.card_queue, CardQueue::Learn));
    }

    #[test]
    fn test_review() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());

        scheduler.card.card_type = CardType::Review;
        scheduler.card.card_queue = CardQueue::Review;
        scheduler.card.due = scheduler.day_today - 8;
        scheduler.card.ease_factor = INITIAL_EASE_FACTOR;
        scheduler.card.reps = 3;
        scheduler.card.lapses = 1;
        scheduler.card.interval = 100;

        let card_copy = scheduler.card.clone();

        // Hard
        scheduler.answer(Choice::Hard);
        assert!(matches!(scheduler.card.card_queue, CardQueue::Review));
        assert!(check_interval(&scheduler.card, 120));
        assert_eq!(
            scheduler.card.due,
            scheduler.day_today + scheduler.card.interval as i64
        );
        assert_eq!(scheduler.card.ease_factor, 2_350);
        assert_eq!(scheduler.card.lapses, 1);
        assert_eq!(scheduler.card.reps, 4);

        // Ok
        scheduler.card = card_copy.clone();
        scheduler.answer(Choice::Ok);
        assert!(matches!(scheduler.card.card_queue, CardQueue::Review));
        assert!(check_interval(&scheduler.card, 260));
        assert_eq!(
            scheduler.card.due,
            scheduler.day_today + scheduler.card.interval as i64
        );
        assert_eq!(scheduler.card.ease_factor, INITIAL_EASE_FACTOR);

        // Easy
        scheduler.card = card_copy.clone();
        scheduler.answer(Choice::Easy);
        assert!(matches!(scheduler.card.card_queue, CardQueue::Review));
        assert!(check_interval(&scheduler.card, 351));
        assert_eq!(
            scheduler.card.due,
            scheduler.day_today + scheduler.card.interval as i64
        );
        assert_eq!(scheduler.card.ease_factor, 2_650);

        // Leech
        scheduler.card = card_copy.clone();
        scheduler.card.lapses = 7;
        scheduler.answer(Choice::Again);
        assert!(matches!(scheduler.card.card_queue, CardQueue::Suspended));
    }

    #[test]
    fn test_spacing_button() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());
        scheduler.card.card_type = CardType::Review;
        scheduler.card.card_queue = CardQueue::Review;
        scheduler.card.due = scheduler.day_today;
        scheduler.card.reps = 1;
        scheduler.card.interval = 1;

        assert_eq!(scheduler.next_interval_string(Choice::Hard), "2d");
        assert_eq!(scheduler.next_interval_string(Choice::Ok), "3d");
        assert_eq!(scheduler.next_interval_string(Choice::Easy), "4d");

        // Hard multiplier = 1, not increase day
        scheduler.config.hard_multiplier = 1.0;
        assert_eq!(scheduler.next_interval_string(Choice::Hard), "1d");
    }

    #[test]
    fn test_bury() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());

        scheduler.bury_card();
        assert!(matches!(scheduler.card.card_queue, CardQueue::Buried));

        scheduler.unbury_card();
        assert!(matches!(scheduler.card.card_queue, CardQueue::New));
    }

    #[test]
    fn test_suspend() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());

        scheduler.card.due = scheduler.day_today;
        scheduler.card.interval = 100;
        scheduler.card.card_queue = CardQueue::Review;
        scheduler.card.card_type = CardType::Review;

        scheduler.answer(Choice::Again);
        assert!(scheduler.card.due > Timestamp::now());
        assert!(matches!(scheduler.card.card_type, CardType::Relearn));
        assert!(matches!(scheduler.card.card_queue, CardQueue::Learn));

        let due = scheduler.card.due;
        scheduler.suspend_card();
        scheduler.unsuspend_card();

        assert!(matches!(scheduler.card.card_type, CardType::Relearn));
        assert!(matches!(scheduler.card.card_queue, CardQueue::Learn));
        assert_eq!(scheduler.card.due, due);
    }

    #[test]
    fn test_reschedule() {
        let mut scheduler =
            Scheduler::new(Card::default(), Config::default(), Timestamp::day_cut_off());

        scheduler.schedule_card_as_review(0, 0);
        assert_eq!(scheduler.card.due, scheduler.day_today);
        assert_eq!(scheduler.card.interval, 1);
        assert!(matches!(scheduler.card.card_queue, CardQueue::Review));
        assert!(matches!(scheduler.card.card_type, CardType::Review));

        scheduler.schedule_card_as_review(1, 1);
        assert_eq!(scheduler.card.due, scheduler.day_today + 1);
        assert_eq!(scheduler.card.interval, 1);

        scheduler.schedule_card_as_new();
        assert_eq!(scheduler.card.due, 0);
        assert!(matches!(scheduler.card.card_queue, CardQueue::New));
        assert!(matches!(scheduler.card.card_type, CardType::New));
    }
}
