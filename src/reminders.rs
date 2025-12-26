use anyhow::Result;
use std::process::Command;

use crate::models::ReminderConfig;

pub fn send_reminder(message: &str, config: &ReminderConfig) -> Result<()> {
    if !config.enabled {
        return Ok(());
    }

    // Try notify-send first (desktop notification)
    if config.use_notify_send {
        let _ = Command::new("notify-send")
            .args(["-i", "appointment", "Todo Reminder", message])
            .status();
    }

    // Try wall (terminal broadcast)
    if config.use_wall {
        let _ = Command::new("wall").arg(message).status();
    }

    Ok(())
}

pub fn check_reminders(config: &ReminderConfig) -> Result<()> {
    if !config.enabled {
        return Ok(());
    }

    let db_path = dirs::home_dir()
        .expect("Cannot determine home directory")
        .join(".todo-queue")
        .join("tasks.db");

    let db = crate::database::Database::new(db_path)?;

    if let Some(task) = db.get_next_task()? {
        let mut message = format!("Current task: {}", task.title);
        if task.is_overdue() {
            message = format!("⚠️ Overdue! {}", message);
        }
        if let Some(due) = task.due_at {
            let now = chrono::Utc::now();
            let duration = due.signed_duration_since(now);
            if duration.num_hours() < 24 && duration.num_hours() > 0 {
                message.push_str(&format!(" (due in {}h)", duration.num_hours()));
            }
        }
        send_reminder(&message, config)?;
    }

    Ok(())
}
