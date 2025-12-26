use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "🟢",
            Priority::Medium => "🟡",
            Priority::High => "🟠",
            Priority::Critical => "🔴",
        }
    }

    pub fn weight(&self) -> u8 {
        match self {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Critical => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub project: Option<String>,
    pub estimated_minutes: Option<u32>,
}

impl Task {
    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_at {
            due < Utc::now() && self.completed_at.is_none()
        } else {
            false
        }
    }

    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderConfig {
    pub enabled: bool,
    pub interval_minutes: u32,
    pub use_notify_send: bool,
    pub use_wall: bool,
}

impl Default for ReminderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 30,
            use_notify_send: true,
            use_wall: false,
        }
    }
}