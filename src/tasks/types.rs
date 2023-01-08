use chrono::NaiveDate;
use heapless::String as HString;

type String = HString<40>;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct Duration {
    weeks: u8,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct Completion {
    date: NaiveDate,
    by: String,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
enum Routine {
    Schedule,
    Interval,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Task {
    name: String,
    kind: Routine,
    participants: heapless::Vec<String, 2>,
    last_completed: Completion,
    duration: Duration,
}
