mod database;
mod models;
mod reminders;
mod ui;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

use database::Database;
use models::{Priority, Task};

#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "Intelligent CLI Task Management System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// Task title
        title: String,
        /// Task description
        #[arg(short, long)]
        description: Option<String>,
        /// Priority (low, medium, high, critical)
        #[arg(short, long, default_value = "medium")]
        priority: String,
        /// Due time (e.g., "2024-12-31 23:59" or "2h", "1d", "1w")
        #[arg(short, long)]
        due: Option<String>,
        /// Project tag
        #[arg(short, long)]
        project: Option<String>,
        /// Task tags (comma separated)
        #[arg(short, long)]
        tags: Option<String>,
        /// Estimated time in minutes
        #[arg(short, long)]
        estimate: Option<u32>,
    },
    /// List all tasks
    List {
        /// Show completed tasks
        #[arg(short, long)]
        completed: bool,
        /// Show all tasks including completed
        #[arg(short, long)]
        all: bool,
        /// Filter by project
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Show next task
    Next,
    /// Complete a task
    Done {
        /// Task index or title
        #[arg(value_name = "INDEX_OR_TITLE")]
        target: Option<String>,
    },
    /// Delete a task
    Delete {
        /// Task index or title
        #[arg(value_name = "INDEX_OR_TITLE")]
        target: String,
    },
    /// Clear all completed tasks
    Clear,
    /// Show task details
    Show {
        /// Task index or title
        #[arg(value_name = "INDEX_OR_TITLE")]
        target: String,
    },
    /// Reset - delete all tasks
    Reset,
    /// Update a task
    Update {
        /// Task index or title
        #[arg(value_name = "INDEX_OR_TITLE")]
        target: String,
        /// New title
        #[arg(short, long)]
        title: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
        /// New priority
        #[arg(short, long)]
        priority: Option<String>,
        /// New due time
        #[arg(short, long)]
        due: Option<String>,
        /// New project
        #[arg(short, long)]
        project: Option<String>,
        /// New tags (comma separated)
        #[arg(short, long)]
        tags: Option<String>,
        /// New estimated time in minutes
        #[arg(short, long)]
        estimate: Option<u32>,
    },
    /// Check reminders
    Remind,
    /// Show statistics
    Stats,
    /// Configure reminder settings
    Config {
        /// Enable or disable reminders
        #[arg(short, long)]
        enabled: Option<bool>,
        /// Reminder interval (e.g., "2h", "30m", or just "60" for minutes)
        #[arg(short, long)]
        interval: Option<String>,
        /// Enable desktop notifications
        #[arg(long)]
        notify: Option<bool>,
        /// Enable terminal wall messages
        #[arg(long)]
        wall: Option<bool>,
        /// Wall quiet hours start (0-23)
        #[arg(long)]
        wall_quiet_start: Option<u32>,
        /// Wall quiet hours end (0-23)
        #[arg(long)]
        wall_quiet_end: Option<u32>,
        /// Start reminders from quiet-end time
        #[arg(long)]
        start_from_quiet_end: Option<bool>,
        /// Show current configuration
        #[arg(short, long)]
        show: bool,
    },
}

fn parse_priority(s: &str) -> Priority {
    match s.to_lowercase().as_str() {
        "low" => Priority::Low,
        "high" => Priority::High,
        "critical" => Priority::Critical,
        _ => Priority::Medium,
    }
}

fn parse_interval(s: &str) -> Result<u32> {
    let s = s.trim().to_lowercase();
    
    if s.ends_with('h') {
        let hours: u32 = s.trim_end_matches('h').parse()
            .map_err(|_| anyhow::anyhow!("Invalid hours format"))?;
        Ok(hours * 60)
    } else if s.ends_with('m') {
        let minutes: u32 = s.trim_end_matches('m').parse()
            .map_err(|_| anyhow::anyhow!("Invalid minutes format"))?;
        Ok(minutes)
    } else {
        // Assume minutes if no suffix
        let minutes: u32 = s.parse()
            .map_err(|_| anyhow::anyhow!("Invalid interval format. Use '2h' for hours or '30m' for minutes"))?;
        Ok(minutes)
    }
}

fn parse_due_time(s: &str) -> Result<Option<DateTime<Utc>>> {
    if s.is_empty() {
        return Ok(None);
    }

    // Parse relative time first
    if s.ends_with('h') {
        let hours: i64 = s.trim_end_matches('h').parse()?;
        return Ok(Some(Utc::now() + Duration::hours(hours)));
    }
    if s.ends_with('d') {
        let days: i64 = s.trim_end_matches('d').parse()?;
        return Ok(Some(Utc::now() + Duration::days(days)));
    }
    if s.ends_with('w') {
        let weeks: i64 = s.trim_end_matches('w').parse()?;
        return Ok(Some(Utc::now() + Duration::weeks(weeks)));
    }

    // Parse absolute time
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(Some(dt.with_timezone(&Utc)));
    }

    // Parse date-only format (YYYY-MM-DD)
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        // Set time to end of day (23:59:59)
        if let Some(naive_datetime) = naive_date.and_hms_opt(23, 59, 59) {
            return Ok(Some(DateTime::from_naive_utc_and_offset(naive_datetime, Utc)));
        }
    }

    // Parse date-time format (YYYY-MM-DD HH:MM)
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        return Ok(Some(DateTime::from_naive_utc_and_offset(naive, Utc)));
    }

    // Parse time-only format (HH:MM) - assume today
    if let Ok(naive_time) = chrono::NaiveTime::parse_from_str(s, "%H:%M") {
        let today = Utc::now().date_naive();
        let naive_datetime = today.and_time(naive_time);
        return Ok(Some(DateTime::from_naive_utc_and_offset(naive_datetime, Utc)));
    }

    Err(anyhow::anyhow!("Cannot parse time format: {}", s))
}

fn is_pure_numeric(s: &str) -> bool {
    // Validate string is numeric only
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

fn find_task_by_index_or_title(tasks: &[Task], target: &str) -> Option<(usize, i64)> {
    // Parse as index first
    if let Ok(index) = target.parse::<usize>() {
        if index >= 1 && index <= tasks.len() {
            return Some((index - 1, tasks[index - 1].id));
        }
    }

    // Search by title
    for (idx, task) in tasks.iter().enumerate() {
        if task.title.eq_ignore_ascii_case(target) {
            return Some((idx, task.id));
        }
    }

    None
}

fn get_db_path() -> PathBuf {
    let mut path = dirs::home_dir().expect("Cannot determine home directory");
    path.push(".todo-queue");
    path.push("tasks.db");
    path
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db = Database::new(get_db_path())?;

    match cli.command {
        Commands::Add {
            title,
            description,
            priority,
            due,
            project,
            tags,
            estimate,
        } => {
            // Check title is not numeric only
            if is_pure_numeric(&title) {
                println!("{} Task title cannot be pure numeric!", "⚠️".yellow());
                println!("   Please use a meaningful name with letters or other characters.");
                return Ok(());
            }

            // Check duplicate task title
            let tasks = db.list_tasks(false)?;
            for task in &tasks {
                if task.title.eq_ignore_ascii_case(&title) {
                    println!("⚠️  Task '{}' already exists!", title);
                    println!("   Use 'todo update \"{}\"' to modify it", title);
                    return Ok(());
                }
            }

            let task = Task {
                id: 0,
                title,
                description,
                priority: parse_priority(&priority),
                created_at: Utc::now(),
                due_at: parse_due_time(&due.unwrap_or_default())?,
                completed_at: None,
                tags: tags
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default(),
                project,
                estimated_minutes: estimate,
            };

            let id = db.add_task(&task)?;
            let tasks = db.list_tasks(false)?;
            let index = tasks.iter().position(|t| t.id == id).map(|i| i + 1).unwrap_or(0);
            println!("✅ {} Task added (Index: {})", task.priority.as_str(), index);
            println!("   {}", task.title.bold());
        }

        Commands::List { completed, all, project } => {
            let mut tasks = db.list_tasks(completed || all)?;

            if let Some(proj) = project {
                tasks.retain(|t| t.project.as_deref() == Some(proj.as_str()));
            }

            if completed {
                ui::print_task_list(&tasks, "📋 All Tasks");
            } else if all {
                ui::print_task_list(&tasks, "📋 All Tasks (Including Completed)");
            } else {
                ui::print_task_list(&tasks, "📋 Pending Tasks");
            }
        }

        Commands::Next => {
            if let Some(task) = db.get_next_task()? {
                println!("\n{}", "🎯 Next Task".bold().underline());
                println!("{}", "=".repeat(50));
                println!("\n{}", ui::format_task(&task, false));

                if task.is_overdue() {
                    println!("\n⚠️  This task is overdue!",);
                }
                println!("\nUse {} to complete this task", "todo done".cyan());
            } else {
                println!("\n{} No pending tasks, enjoy your life! 🎉", "✨".bold());
            }
        }

        Commands::Done { target } => {
            let tasks = db.list_tasks(false)?;
            let task_id = if let Some(ref t) = target {
                if let Some((_, id)) = find_task_by_index_or_title(&tasks, t) {
                    id
                } else if let Some(task) = db.get_next_task()? {
                    task.id
                } else {
                    println!("{} No pending tasks", "⚠️".yellow());
                    return Ok(());
                }
            } else if let Some(task) = db.get_next_task()? {
                task.id
            } else {
                println!("{} No pending tasks", "⚠️".yellow());
                return Ok(());
            };

            if db.complete_task(task_id)? {
                if let Some(task) = db.get_task(task_id)? {
                    println!("✅ Task completed!");
                    println!("   {}", task.title.bold());
                }
            } else {
                println!("{} Task not found or already completed", "⚠️".yellow());
            }
        }

        Commands::Delete { target } => {
            let tasks = db.list_tasks(false)?;
            if let Some((_, task_id)) = find_task_by_index_or_title(&tasks, &target) {
                if db.delete_task(task_id)? {
                    println!("🗑️  Task deleted permanently");
                } else {
                    println!("{} Failed to delete task", "⚠️".yellow());
                }
            } else {
                println!("{} Task not found. Use 'todo list' to see valid indices or titles.", "⚠️".yellow());
            }
        }

        Commands::Clear => {
            let count = db.clear_completed()?;
            println!("🧹 Cleared {} completed tasks", count);
        }

        Commands::Reset => {
            // Display current task count
            let tasks = db.list_tasks(true)?;
            let total = tasks.len();
            let completed = tasks.iter().filter(|t| t.is_completed()).count();
            let pending = total - completed;

            println!("\n{}", "⚠️  WARNING: This will delete ALL tasks!".bold().red());
            println!("{}", "=".repeat(50));
            println!("Total tasks: {}", total);
            println!("  - Pending: {}", pending);
            println!("  - Completed: {}", completed);
            println!();

            // Ask for confirmation
            print!("Are you sure you want to delete ALL tasks? (type 'yes' to confirm): ");
            std::io::Write::flush(&mut std::io::stdout())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "yes" {
                let count = db.reset_all()?;
                println!("\n✅ Deleted {} tasks from database", count);
            } else {
                println!("\n❌ Reset cancelled");
            }
        }

        Commands::Show { target } => {
            let tasks = db.list_tasks(true)?;
            if let Some((idx, _)) = find_task_by_index_or_title(&tasks, &target) {
                let task = &tasks[idx];
                println!("\n{}", "📝 Task Details".bold().underline());
                println!("{}", "=".repeat(50));
                println!("\n{}", ui::format_task(task, true));
                println!("\nCreated: {}", task.created_at.format("%Y-%m-%d %H:%M:%S"));
                if let Some(due) = task.due_at {
                    println!("Due: {}", due.format("%Y-%m-%d %H:%M:%S"));
                }
                if let Some(completed) = task.completed_at {
                    println!("Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));
                }
            } else {
                println!("{} Task not found. Use 'todo list --all' to see all valid indices or titles.", "⚠️".yellow());
            }
        }

        Commands::Update {
            target,
            title,
            description,
            priority,
            due,
            project,
            tags,
            estimate,
        } => {
            let tasks = db.list_tasks(false)?;
            if let Some((_, task_id)) = find_task_by_index_or_title(&tasks, &target) {
                if let Some(mut task) = db.get_task(task_id)? {
                    // Update only provided fields
                    if let Some(new_title) = title {
                        task.title = new_title;
                    }
                    if let Some(new_description) = description {
                        task.description = Some(new_description);
                    }
                    if let Some(new_priority) = priority {
                        task.priority = parse_priority(&new_priority);
                    }
                    if let Some(new_due) = due {
                        task.due_at = parse_due_time(&new_due)?;
                    }
                    if let Some(new_project) = project {
                        task.project = Some(new_project);
                    }
                    if let Some(new_tags) = tags {
                        task.tags = new_tags.split(',').map(|s| s.trim().to_string()).collect();
                    }
                    if let Some(new_estimate) = estimate {
                        task.estimated_minutes = Some(new_estimate);
                    }

                    if db.update_task(task_id, &task)? {
                        println!("✅ Task updated");
                        println!("   {}", task.title.bold());
                    } else {
                        println!("{} Failed to update task", "⚠️".yellow());
                    }
                }
            } else {
                println!("{} Task not found. Use 'todo list' to see valid indices or titles.", "⚠️".yellow());
            }
        }

        Commands::Remind => {
            let config = db.get_config()?;
            reminders::check_reminders(&config)?;
        }

        Commands::Config {
            enabled,
            interval,
            notify,
            wall,
            wall_quiet_start,
            wall_quiet_end,
            start_from_quiet_end,
            show,
        } => {
            let mut config = db.get_config()?;
            let mut changed = false;

            // Show current configuration
            if show || (enabled.is_none() && interval.is_none() && notify.is_none() && wall.is_none() && wall_quiet_start.is_none() && wall_quiet_end.is_none() && start_from_quiet_end.is_none()) {
                println!("\n{}", "🔧 Current Reminder Configuration".bold().underline());
                println!("{}", "═".repeat(50));
                println!("  Enabled: {}", if config.enabled { "✅ Yes" } else { "❌ No" });
                
                // Format interval
                let hours = config.interval_minutes / 60;
                let mins = config.interval_minutes % 60;
                if hours > 0 && mins > 0 {
                    println!("  Interval: {}h {}m", hours, mins);
                } else if hours > 0 {
                    println!("  Interval: {}h", hours);
                } else {
                    println!("  Interval: {}m", mins);
                }
                
                println!("  Desktop Notifications: {}", if config.use_notify_send { "✅ Yes" } else { "❌ No" });
                println!("  Terminal Wall Messages: {}", if config.use_wall { "✅ Yes" } else { "❌ No" });
                if config.use_wall {
                    println!("  Wall Quiet Hours: {}:00 - {}:00 (no wall messages)", 
                             config.wall_quiet_start_hour, config.wall_quiet_end_hour);
                    if config.start_from_quiet_end {
                        println!("  Start Time: Reminders start from quiet-end time");
                    }
                }
                println!();
                println!("To change configuration, use:");
                println!("  {} --enabled true/false", "todo config".cyan());
                println!("  {} --interval <time> (e.g., '2h', '30m', '60')", "todo config".cyan());
                println!("  {} --notify true/false", "todo config".cyan());
                println!("  {} --wall true/false", "todo config".cyan());
                println!("  {} --wall-quiet-start <hour> (0-23)", "todo config".cyan());
                println!("  {} --wall-quiet-end <hour> (0-23)", "todo config".cyan());
                println!("  {} --start-from-quiet-end true/false", "todo config".cyan());
                return Ok(());
            }

            // Update configuration
            if let Some(e) = enabled {
                config.enabled = e;
                changed = true;
                println!("✅ Reminders {}", if e { "enabled" } else { "disabled" });
            }

            if let Some(i) = interval {
                config.interval_minutes = parse_interval(&i)?;
                changed = true;
                
                // Format for display
                let hours = config.interval_minutes / 60;
                let mins = config.interval_minutes % 60;
                if hours > 0 && mins > 0 {
                    println!("✅ Reminder interval set to {}h {}m", hours, mins);
                } else if hours > 0 {
                    println!("✅ Reminder interval set to {}h", hours);
                } else {
                    println!("✅ Reminder interval set to {}m", mins);
                }
            }

            if let Some(n) = notify {
                config.use_notify_send = n;
                changed = true;
                println!("✅ Desktop notifications {}", if n { "enabled" } else { "disabled" });
            }

            if let Some(w) = wall {
                config.use_wall = w;
                changed = true;
                println!("✅ Terminal wall messages {}", if w { "enabled" } else { "disabled" });
            }

            if let Some(start) = wall_quiet_start {
                config.wall_quiet_start_hour = start.min(23);
                changed = true;
                println!("✅ Wall quiet start hour set to {}:00", start);
            }

            if let Some(end) = wall_quiet_end {
                config.wall_quiet_end_hour = end.min(23);
                changed = true;
                println!("✅ Wall quiet end hour set to {}:00", end);
            }

            if let Some(s) = start_from_quiet_end {
                config.start_from_quiet_end = s;
                changed = true;
                if s {
                    println!("✅ Reminders will start from quiet-end time");
                } else {
                    println!("✅ Reminders will use fixed interval");
                }
            }

            if changed {
                db.save_config(&config)?;
                println!();
                println!("⚠️  To apply changes, run the following commands:");
                println!("   1. systemctl --user daemon-reload");
                println!("   2. systemctl --user restart todo-queue.timer");
            }
        }

        Commands::Stats => {
            let tasks = db.list_tasks(true)?;
            ui::print_stats(&tasks);
        }
    }

    Ok(())
}
