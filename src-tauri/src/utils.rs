use std::{
    hash::{Hash, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Datelike, Utc};
use chrono_tz::Tz;

pub fn with_local_timezone(date_time: DateTime<Utc>) -> DateTime<Tz> {
    let tz_str = iana_time_zone::get_timezone().unwrap_or(chrono_tz::UTC.to_string());
    let timezone: Tz = tz_str.parse().unwrap_or(Tz::UTC);
    date_time.with_timezone(&timezone)
}

pub fn get_current_datetime() -> DateTime<Utc> {
    let dt = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    DateTime::from_timestamp(dt.as_secs() as i64, dt.subsec_nanos()).unwrap()
}

pub fn get_current_date() -> String {
    let today = Utc::now();
    format!("{}-{}-{}", today.year(), today.month(), today.day())
}

use rand::{thread_rng, Rng};

pub fn gen_rand_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn gen_rand_number() -> u32 {
    let time_ms = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    time_ms.hash(&mut hasher);
    hasher.finish() as u32
}
