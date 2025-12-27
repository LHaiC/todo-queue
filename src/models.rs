use chrono::{DateTime, Timelike, Utc};
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

    #[allow(dead_code)]
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
    pub wall_quiet_start_hour: u32,  // Start hour for wall quiet period (0-23)
    pub wall_quiet_end_hour: u32,    // End hour for wall quiet period (0-23)
    pub start_from_quiet_end: bool,  // Start reminders from quiet-end time
}

impl Default for ReminderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 180,  // Default: 3 hours
            use_notify_send: true,
            use_wall: false,
            wall_quiet_start_hour: 18,  // 6 PM
            wall_quiet_end_hour: 9,     // 9 AM
            start_from_quiet_end: false,
        }
    }
}

impl ReminderConfig {
    /// Check if current time is within wall quiet hours
    pub fn is_wall_quiet_hours(&self) -> bool {
        let now = Utc::now();
        let hour = now.hour() as u32;
        
        // 处理跨天情况（例如：18:00 - 09:00）
        if self.wall_quiet_start_hour > self.wall_quiet_end_hour {
            // 跨天：18:00 到 09:00
            hour >= self.wall_quiet_start_hour || hour < self.wall_quiet_end_hour
        } else {
            // 同一天：09:00 到 18:00
            hour >= self.wall_quiet_start_hour && hour < self.wall_quiet_end_hour
        }
    }
}
