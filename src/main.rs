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
        /// Filter by project
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Show next task
    Next,
    /// Complete a task
    Done {
        /// Task ID, complete next task if not specified
        id: Option<i64>,
    },
    /// Delete a task
    Delete {
        /// Task ID
        id: i64,
    },
    /// Clear all completed tasks
    Clear,
    /// Show task details
    Show {
        /// Task ID
        id: i64,
    },
    /// Check reminders
    Remind,
    /// Show statistics
    Stats,
}

fn parse_priority(s: &str) -> Priority {
    match s.to_lowercase().as_str() {
        "low" => Priority::Low,
        "high" => Priority::High,
        "critical" => Priority::Critical,
        _ => Priority::Medium,
    }
}

fn parse_due_time(s: &str) -> Result<Option<DateTime<Utc>>> {
    if s.is_empty() {
        return Ok(None);
    }

    // Try relative time first
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

    // Try absolute time
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(Some(dt.with_timezone(&Utc)));
    }

    // Try common formats
    for format in &["%Y-%m-%d %H:%M", "%Y-%m-%d", "%H:%M"] {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, format) {
            return Ok(Some(DateTime::from_naive_utc_and_offset(naive, Utc)));
        }
    }

    Err(anyhow::anyhow!("Cannot parse time format: {}", s))
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
            let task = Task {
                id: 0,
                title,
                description,
                priority: parse_priority(&priority),
                created_at: Utc::now(),
                due_at: parse_due_time(&due.unwrap_or_default())?,
                completed_at: None,
                tags: tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default(),
                project,
                estimated_minutes: estimate,
            };

            let id = db.add_task(&task)?;
            println!("✅ {} Task added (ID: {})", task.priority.as_str(), id);
            println!("   {}", task.title.bold());
        }

        Commands::List { completed, project } => {
            let mut tasks = db.list_tasks(completed)?;

            if let Some(proj) = project {
                tasks.retain(|t| t.project.as_deref() == Some(proj.as_str()));
            }

            if completed {
                ui::print_task_list(&tasks, "📋 All Tasks");
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
                    println!("\n⚠️  This task is overdue!", );
                }
                println!("\nUse {} to complete this task", "todo done".cyan());
            } else {
                println!("\n{} No pending tasks, enjoy your life! 🎉", "✨".bold());
            }
        }

        Commands::Done { id } => {
            let task_id = if let Some(id) = id {
                id
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

        Commands::Delete { id } => {
            if db.delete_task(id)? {
                println!("🗑️  Task deleted (ID: {})", id);
            } else {
                println!("{} Task not found", "⚠️".yellow());
            }
        }

        Commands::Clear => {
            let count = db.clear_completed()?;
            println!("🧹 Cleared {} completed tasks", count);
        }

        Commands::Show { id } => {
            if let Some(task) = db.get_task(id)? {
                println!("\n{}", "📝 Task Details".bold().underline());
                println!("{}", "=".repeat(50));
                println!("\n{}", ui::format_task(&task, true));
                println!("\nCreated: {}", task.created_at.format("%Y-%m-%d %H:%M:%S"));
                if let Some(due) = task.due_at {
                    println!("Due: {}", due.format("%Y-%m-%d %H:%M:%S"));
                }
                if let Some(completed) = task.completed_at {
                    println!("Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));
                }
            } else {
                println!("{} Task not found", "⚠️".yellow());
            }
        }

        Commands::Remind => {
            let config = db.get_config()?;
            reminders::check_reminders(&config)?;
        }

        Commands::Stats => {
            let tasks = db.list_tasks(true)?;
            ui::print_stats(&tasks);
        }
    }

    Ok(())
}