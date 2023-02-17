// SPDX-FileCopyrightText: 2023 Jonathan Frere
//
// SPDX-License-Identifier: MPL-2.0

use chrono::{Duration, NaiveDate};
use sqlx::Sqlite;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize, sqlx::Type)]
pub enum Routine {
    Schedule,
    Interval,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Deserialize,
    serde::Serialize,
    sqlx::Encode,
    sqlx::Decode,
)]
pub struct TaskId(i32);

impl sqlx::Type<Sqlite> for TaskId {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <i32 as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

#[cfg(test)]
impl From<i32> for TaskId {
    fn from(value: i32) -> Self {
        TaskId(value)
    }
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
    pub id: TaskId,
    pub name: String,
    pub kind: Routine,
    pub assigned_to: String,
    pub deadline: Deadline,
    pub length_days: u16,
    pub last_completed: NaiveDate,
    pub participants: Vec<String>,
}
