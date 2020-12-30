mod card;

#[derive(PartialEq)]
pub enum CardType {
    New = 0,
    Learn = 1,
    Review = 2,
    Relearn = 3,
}

#[derive(PartialEq)]
pub enum CardQueue {
    New = 0, // due is the order cards are shown in
    Learn = 1, // due is a unix timestamp
    Review = 2, // due is days since epoch
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
