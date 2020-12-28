use std::time;

pub struct Timestamp(i64);

impl Timestamp {
    pub fn now() -> i64 {
        now().as_secs() as i64
    }
}

pub fn now() -> time::Duration {
    time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)
        .unwrap()
}

