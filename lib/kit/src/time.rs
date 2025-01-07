use chrono::prelude::*;
use floating_duration::TimeFormat;
use ms_converter::get_duration_by_postfix;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn utc_now() -> String {
    let date = Utc::now();
    date.format("%m:%d:%Y-%H:%M:%S").to_string()
}

pub fn current_millis() -> i64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis().try_into().unwrap()
}

pub fn simple_date() -> String {
    let date = Local::now();
    date.format("%m-%d-%Y").to_string()
}

pub fn human_time(ms: i64) -> String {
    let diff = ms - current_millis();
    if diff > 0 {
        get_duration_by_postfix(diff, " hours").unwrap()
    } else {
        String::from("Expired")
    }
}

pub fn time_format(duration: Duration) -> TimeFormat<Duration> {
    TimeFormat(duration)
}
