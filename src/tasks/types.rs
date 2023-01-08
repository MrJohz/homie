use chrono::Duration;

type String = heapless::String<40>;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum Routine {
    Schedule,
    Interval,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum Deadline {
    Upcoming(u8),
    Overdue(u8),
}

impl From<Duration> for Deadline {
    fn from(duration: Duration) -> Self {
        let days = duration.num_days();
        if days >= 0 {
            Self::Upcoming(days as u8)
        } else {
            Self::Overdue(days as u8)
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Task {
    pub name: String,
    pub kind: Routine,
    pub assigned_to: String,
    pub deadline: Deadline,
}