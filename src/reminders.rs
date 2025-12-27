use anyhow::Result;
use std::process::Command;

use crate::models::ReminderConfig;

pub fn send_reminder(message: &str, config: &ReminderConfig) -> Result<()> {
    if !config.enabled {
        return Ok(());
    }

    // Always print to stdout for debugging
    println!("📢 {}", message);

    // Try notify-send first (desktop notification) - no quiet hours
    if config.use_notify_send {
        let _ = Command::new("notify-send")
            .args(["-i", "appointment", "Todo Reminder", message])
            .status();
    }

    // Try wall (terminal broadcast) - respect quiet hours
    if config.use_wall {
        // Check if within quiet hours
        if config.is_wall_quiet_hours() {
            println!("🔇 Wall message suppressed (quiet hours: {}:00 - {}:00)", 
                     config.wall_quiet_start_hour, config.wall_quiet_end_hour);
        } else {
            let _ = Command::new("wall").arg(message).status();
        }
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

    // Get all pending tasks
    let tasks = db.list_tasks(false)?;
    
    if tasks.is_empty() {
        return Ok(());
    }

    // 构建提醒消息，包含所有任务
    let mut message_parts = Vec::new();
    
    // 添加标题
    if tasks.len() == 1 {
        message_parts.push(format!("Current task: {}", tasks[0].title));
    } else {
        message_parts.push(format!("You have {} pending tasks:", tasks.len()));
    }
    
    // 添加每个任务的信息
    for (idx, task) in tasks.iter().enumerate() {
        let task_info = if task.is_overdue() {
            format!("⚠️ [{}] {} (OVERDUE)", idx + 1, task.title)
        } else if let Some(due) = task.due_at {
            let now = chrono::Utc::now();
            let duration = due.signed_duration_since(now);
            if duration.num_hours() < 24 && duration.num_hours() > 0 {
                format!("  [{}] {} (due in {}h)", idx + 1, task.title, duration.num_hours())
            } else if duration.num_hours() <= 0 {
                format!("⚠️ [{}] {} (OVERDUE)", idx + 1, task.title)
            } else {
                format!("  [{}] {}", idx + 1, task.title)
            }
        } else {
            format!("  [{}] {}", idx + 1, task.title)
        };
        message_parts.push(task_info);
    }

    let message = message_parts.join("\n");
    send_reminder(&message, config)?;

    Ok(())
}
