use std::time;

use chrono::{DateTime, Duration, FixedOffset, Local, TimeZone, Utc};

pub struct Timestamp(i64);

impl Timestamp {
    pub fn now() -> i64 {
        now().as_secs() as i64
    }

    pub fn day_cut_off() -> i64 {
        get_next_day(
            now().as_secs() as i64,
            local_minutes_west_for_stamp(Utc::now().timestamp()),
            4,
        )
        .timestamp()
    }
}

pub fn now() -> time::Duration {
    time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)
        .unwrap()
}

/// - now_secs is a timestamp of the current time
/// - now_mins_west is the current offset west of UTC
/// - rollover_hour is the hour of the day the rollover happens (eg 4 for 4am)
pub fn get_next_day(now_secs: i64, now_mins_west: i32, rollover_hour: u8) -> DateTime<FixedOffset> {
    let now_datetime = fixed_offset_from_minutes(now_mins_west).timestamp(now_secs, 0);
    let today = now_datetime.date();

    // rollover
    let rollover_today_datetime = today.and_hms(rollover_hour as u32, 0, 0);
    let rollover_passed = rollover_today_datetime <= now_datetime;

    if rollover_passed {
        rollover_today_datetime + Duration::days(1)
    } else {
        rollover_today_datetime
    }
}

fn fixed_offset_from_minutes(minutes_west: i32) -> FixedOffset {
    let bounded_minutes = minutes_west.max(-23 * 60).min(23 * 60);
    FixedOffset::west(bounded_minutes * 60)
}

fn local_minutes_west_for_stamp(stamp: i64) -> i32 {
    Local.timestamp(stamp, 0).offset().utc_minus_local() / 60
}
