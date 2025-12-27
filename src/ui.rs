use crate::models::Task;
use chrono::Utc;
use colored::*;

pub fn format_task(task: &Task, show_id: bool) -> String {
    let id_str = if show_id {
        format!("[{}] ", task.id)
    } else {
        String::new()
    };

    let priority_icon = task.priority.as_str();
    
    // Add completion badge
    let status_badge = if task.is_completed() {
        "✅".green().to_string()
    } else {
        "🔲".dimmed().to_string()
    };
    
    let title = if task.is_completed() {
        task.title.strikethrough().dimmed()
    } else {
        task.title.bold()
    };

    let mut parts = vec![format!("{}{}{} {}", id_str, status_badge, priority_icon, title)];

    if let Some(ref desc) = task.description {
        parts.push(format!("   {}", desc.dimmed()));
    }

    if let Some(ref project) = task.project {
        parts.push(format!("   📁 {}", project.cyan()));
    }

    if !task.tags.is_empty() {
        let tags: Vec<String> = task
            .tags
            .iter()
            .map(|t| format!("#{}", t.green()))
            .collect();
        parts.push(format!("   {}", tags.join(" ")));
    }

    if let Some(due) = task.due_at {
        let now = Utc::now();
        let duration = due.signed_duration_since(now);
        let due_str = if duration.num_hours() < 0 {
            format!("   ⚠️ Overdue by {}", format_duration(duration.abs()))
        } else if duration.num_hours() < 24 {
            format!("   ⏰ Due in {}", format_duration(duration))
        } else {
            format!("   📅 {}", due.format("%Y-%m-%d %H:%M"))
        };
        parts.push(due_str.yellow().to_string());
    }

    if let Some(mins) = task.estimated_minutes {
        parts.push(format!("   ⏱️  Est. {} min", mins));
    }

    // Add completion timestamp if task is done
    if let Some(completed) = task.completed_at {
        parts.push(format!("   ✨ Completed: {}", completed.format("%Y-%m-%d %H:%M").to_string().green()));
    }

    parts.join("\n")
}

fn format_duration(duration: chrono::Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;

    if days > 0 {
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

pub fn print_task_list(tasks: &[Task], title: &str) {
    println!("\n{}", title.bold().underline());
    println!("{}", "═".repeat(60));

    if tasks.is_empty() {
        println!("\n  {} No tasks found\n", "✨".dimmed());
    } else {
        for (index, task) in tasks.iter().enumerate() {
            // Use sequential index instead of database ID
            let display_task = format_task_with_index(task, index + 1);
            println!("\n{}", display_task);
            if index < tasks.len() - 1 {
                println!("{}", "─".repeat(60).dimmed());
            }
        }
        println!("\n{}", "═".repeat(60));
        println!("  Total: {} task{}\n", tasks.len(), if tasks.len() != 1 { "s" } else { "" });
    }
}

fn format_task_with_index(task: &Task, index: usize) -> String {
    let index_str = format!("[{}] ", index);
    let priority_icon = task.priority.as_str();
    
    // Add completion badge
    let status_badge = if task.is_completed() {
        "✅".green().to_string()
    } else {
        "🔲".dimmed().to_string()
    };
    
    let title = if task.is_completed() {
        task.title.strikethrough().dimmed()
    } else {
        task.title.bold()
    };

    let mut parts = vec![format!("{}{}{} {}", index_str, status_badge, priority_icon, title)];

    if let Some(ref desc) = task.description {
        parts.push(format!("   {}", desc.dimmed()));
    }

    if let Some(ref project) = task.project {
        parts.push(format!("   📁 {}", project.cyan()));
    }

    if !task.tags.is_empty() {
        let tags: Vec<String> = task.tags.iter().map(|t| format!("#{}", t.green())).collect();
        parts.push(format!("   {}", tags.join(" ")));
    }

    if let Some(due) = task.due_at {
        let now = Utc::now();
        let duration = due.signed_duration_since(now);
        let due_str = if duration.num_hours() < 0 {
            format!("   ⚠️ Overdue by {}", format_duration(duration.abs()))
        } else if duration.num_hours() < 24 {
            format!("   ⏰ Due in {}", format_duration(duration))
        } else {
            format!("   📅 {}", due.format("%Y-%m-%d %H:%M"))
        };
        parts.push(due_str.yellow().to_string());
    }

    if let Some(mins) = task.estimated_minutes {
        parts.push(format!("   ⏱️  Est. {} min", mins));
    }

    // Add completion timestamp if task is done
    if let Some(completed) = task.completed_at {
        parts.push(format!("   ✨ Completed: {}", completed.format("%Y-%m-%d %H:%M").to_string().green()));
    }

    parts.join("\n")
}

pub fn print_stats(tasks: &[Task]) {
    let total = tasks.len();
    let completed = tasks.iter().filter(|t| t.is_completed()).count();
    let pending = total - completed;
    let overdue = tasks.iter().filter(|t| t.is_overdue()).count();

    println!("\n{}", "📊 Statistics".bold());
    println!("{} Total: {}", "•".dimmed(), total);
    println!("{} Pending: {}", "•".dimmed(), pending.to_string().yellow());
    println!(
        "{} Completed: {}",
        "•".dimmed(),
        completed.to_string().green()
    );
    if overdue > 0 {
        println!(
            "{} Overdue: {}",
            "•".dimmed(),
            overdue.to_string().red().bold()
        );
    }
    println!();
}
