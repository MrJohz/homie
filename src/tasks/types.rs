use chrono::NaiveDate;
use heapless::String as HString;

type String = HString<40>;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub(super) struct Duration {
    pub(super) weeks: u8,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct Completion {
    pub(super) date: NaiveDate,
    pub(super) by: String,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub(super) enum Routine {
    Schedule,
    Interval,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Task {
    pub(super) name: String,
    pub(super) kind: Routine,
    pub(super) participants: heapless::Vec<String, 2>,
    pub(super) last_completed: Completion,
    pub(super) duration: Duration,
}

impl Task {
    pub fn assigned_to(&self) -> &str {
        let mut participants_iter = self.participants.iter();
        while let Some(person) = participants_iter.next() {
            if person == &self.last_completed.by {
                return participants_iter.next().unwrap_or(&self.participants[0]);
            }
        }

        "oh no"
    }
}
