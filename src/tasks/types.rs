use chrono::{Duration, NaiveDate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize, sqlx::Type)]
pub enum Routine {
    Schedule,
    Interval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Deadline {
    Upcoming(u16),
    Overdue(u16),
}

impl From<Duration> for Deadline {
    fn from(duration: Duration) -> Self {
        let days = duration.num_days();
        if days >= 0 {
            Self::Upcoming(days as u16)
        } else {
            Self::Overdue((-days) as u16)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Task {
    pub name: String,
    pub kind: Routine,
    pub assigned_to: String,
    pub deadline: Deadline,
    pub length_days: u16,
    pub last_completed: NaiveDate,
    pub participants: Vec<String>,
}
