---
marp: true
theme: default
paginate: true
---

<!-- _class: lead -->

# Todo Queue Architecture

## A Rust-Based Intelligent CLI Task Management System

---

# Table of Contents

1. Project Overview
2. Architecture Diagram
3. Module Breakdown
4. Dependencies & Usage
5. Data Flow
6. System Integration

---

# Project Overview

**Todo Queue** is a command-line task management system built with Rust.

## Key Characteristics

- **Language**: Rust 2021 Edition
- **Storage**: SQLite database
- **CLI Framework**: clap
- **Reminder System**: systemd timers
- **Architecture**: Modular, clean separation of concerns

---

# Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Layer                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  main.rs (clap Parser)                                │  │
│  │  • Command parsing                                    │  │
│  │  • Argument validation                                │  │
│  │  • Routing to handlers                                │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Business Logic Layer                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Database    │  │     UI       │  │  Reminders   │      │
│  │  (SQLite)    │  │  (Display)   │  │  (Notify)    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       Data Layer                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  models.rs                                            │  │
│  │  • Task struct                                        │  │
│  │  • Priority enum                                      │  │
│  │  • ReminderConfig                                     │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Storage Layer                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  SQLite Database (~/.todo-queue/tasks.db)            │  │
│  │  • Tasks table                                        │  │
│  │  • Config table                                       │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

# Module Breakdown: main.rs

**Purpose**: Entry point and CLI command routing

## Key Components

```rust
// CLI Parser using clap
#[derive(Parser)]
struct Cli {
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add, List, Next, Done, Delete, Clear, Show, Remind, Stats
}
```

## Functions

- `parse_priority()`: Convert string to Priority enum
- `parse_due_time()`: Parse relative/absolute time formats
- `get_db_path()`: Resolve database path
- `main()`: Command routing orchestration

---

# Module Breakdown: models.rs

**Purpose**: Data structures and domain models

## Key Models

```rust
pub enum Priority {
    Low,      // 🟢
    Medium,   // 🟡
    High,     // 🟠
    Critical, // 🔴
}

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
```

---

# Module Breakdown: database.rs

**Purpose**: SQLite database operations

## Key Methods

| Method | Purpose |
|--------|---------|
| `new()` | Open/create database connection |
| `add_task()` | Insert new task |
| `get_task()` | Retrieve task by ID |
| `list_tasks()` | List all/pending tasks |
| `get_next_task()` | Get highest priority pending task |
| `complete_task()` | Mark task as completed |
| `delete_task()` | Remove task |
| `clear_completed()` | Delete all completed tasks |

## SQL Operations

```rust
// Example: Add task
INSERT INTO tasks (title, description, priority, ...)
VALUES (?1, ?2, ?3, ...)

// Example: Get next task
SELECT * FROM tasks WHERE completed_at IS NULL
ORDER BY priority DESC, due_at ASC, created_at ASC
LIMIT 1
```

---

# Module Breakdown: ui.rs

**Purpose**: Terminal UI formatting and display

## Functions

```rust
format_task(task, show_id)  // Format task for display
format_duration(duration)  // Convert duration to readable string
print_task_list(tasks, title)  // Display task list
print_stats(tasks)  // Display statistics
```

## Output Format

```
[3] 🔴 Fix critical bug
   ⏰ Due in 1h 57m
────────────────────────────────────────
```

---

# Module Breakdown: reminders.rs

**Purpose**: Reminder notification logic

## Functions

```rust
send_reminder(message, config)  // Send notifications
check_reminders(config)  // Check and send reminders
```

## Notification Methods

1. **notify-send**: Desktop notifications (Linux)
2. **wall**: Terminal broadcast (server environments)

---

# Dependencies Overview

## Core Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.5 | CLI argument parsing |
| `rusqlite` | 0.32 | SQLite database bindings |
| `serde` | 1.0 | Serialization/deserialization |
| `chrono` | 0.4 | Date/time handling |

## UI Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `colored` | 2.1 | Terminal colors |
| `crossterm` | 0.28 | Terminal control |
| `ratatui` | 0.28 | TUI framework (future use) |

## Utility Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `anyhow` | 1.0 | Error handling |
| `dirs` | 5.0 | Platform directories |

---

# Dependency: clap

**Purpose**: Command-line argument parsing

## Usage in Project

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "todo")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        title: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long, default_value = "medium")]
        priority: String,
        // ...
    },
    // ... other commands
}
```

## Benefits

- Automatic `--help` generation
- Type-safe argument parsing
- Subcommand support
- Validation

---

# Dependency: rusqlite

**Purpose**: SQLite database bindings for Rust

## Usage in Project

```rust
use rusqlite::{params, Connection, OptionalExtension};

// Open database
let conn = Connection::open(path)?;

// Execute query
conn.execute(
    "INSERT INTO tasks (title, ...) VALUES (?1, ?2, ...)",
    params![title, ...],
)?;

// Query with result
let task = stmt.query_row(params![id], |row| {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        // ...
    })
}).optional()?;
```

## Features Used

- `bundled`: Embed SQLite library
- `chrono`: DateTime support

---

# Dependency: chrono

**Purpose**: Date and time handling

## Usage in Project

```rust
use chrono::{DateTime, Duration, Utc};

// Current time
let now = Utc::now();

// Parse time
let dt = DateTime::parse_from_rfc3339("2024-12-31T23:59:00Z")?;

// Calculate duration
let duration = due.signed_duration_since(now);

// Format time
println!("{}", dt.format("%Y-%m-%d %H:%M:%S"));
```

## Key Operations

- Time parsing (RFC3339, custom formats)
- Duration calculations
- Time formatting
- Timezone handling (UTC)

---

# Dependency: serde

**Purpose**: Serialization framework

## Usage in Project

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low, Medium, High, Critical,
}

// Serialize to JSON
let json = serde_json::to_string(&priority)?;

// Deserialize from JSON
let priority: Priority = serde_json::from_str(&json)?;
```

## Purpose in Project

- Store enums in SQLite as JSON
- Configuration serialization

---

# Dependency: colored

**Purpose**: Terminal color output

## Usage in Project

```rust
use colored::*;

println!("{}", "Error".red());
println!("{}", "Success".green());
println!("{}", "Warning".yellow());
println!("{}", "{}", title.bold());
println!("{}", "dimmed".dimmed());
```

## Output Examples

```
✅ Task added
⚠️ Task not found
📋 Pending Tasks
```

---

# Dependency: anyhow

**Purpose**: Error handling

## Usage in Project

```rust
use anyhow::{Context, Result};

fn get_db_path() -> PathBuf {
    let mut path = dirs::home_dir()
        .expect("Cannot determine home directory");
    path.push(".todo-queue");
    path.push("tasks.db");
    path
}

fn parse_due_time(s: &str) -> Result<Option<DateTime<Utc>>> {
    let hours: i64 = s.trim_end_matches('h').parse()
        .context("Failed to parse hours")?;
    Ok(Some(Utc::now() + Duration::hours(hours)))
}
```

## Benefits

- Easy error propagation with `?`
- Context for errors
- Any error type support

---

# Dependency: dirs

**Purpose**: Platform-specific directories

## Usage in Project

```rust
use dirs::home_dir;

// Get home directory
let home = dirs::home_dir()
    .expect("Cannot determine home directory");

let db_path = home.join(".todo-queue").join("tasks.db");
```

## Supported Platforms

- Linux: `~/.todo-queue/`
- macOS: `~/.todo-queue/`
- Windows: `C:\Users\<user>\.todo-queue\`

---

# Data Flow: Add Task

```
User Input
    │
    ▼
┌─────────────────┐
│  clap Parser    │
│  (main.rs)      │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Parse Args     │
│  • title        │
│  • priority     │
│  • due time     │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Create Task    │
│  (models.rs)    │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Insert to DB   │
│  (database.rs)  │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Format Output  │
│  (ui.rs)        │
└─────────────────┘
    │
    ▼
User Output
```

---

# Data Flow: List Tasks

```
User Command
    │
    ▼
┌─────────────────┐
│  Query DB       │
│  (database.rs)  │
│  SELECT * FROM  │
│  tasks WHERE... │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Deserialize    │
│  to Task structs│
│  (models.rs)    │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Sort by        │
│  priority, due  │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Format Display │
│  (ui.rs)        │
└─────────────────┘
    │
    ▼
User Output
```

---

# Data Flow: Reminder

```
systemd Timer
    │ (every 30 min)
    ▼
┌─────────────────┐
│  Execute        │
│  todo remind    │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Get Next Task  │
│  (database.rs)  │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Check Overdue  │
│  Calculate Time │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Send Notify    │
│  (reminders.rs) │
│  • notify-send  │
│  • wall         │
└─────────────────┘
    │
    ▼
User Notification
```

---

# System Integration: systemd

## Service Files

### todo-queue.service
```ini
[Unit]
Description=Todo Queue Reminder Service

[Service]
Type=oneshot
ExecStart=/path/to/todo remind
```

### todo-queue.timer
```ini
[Unit]
Description=Todo Queue Reminder Timer

[Timer]
OnCalendar=*:0/30
Persistent=true

[Install]
WantedBy=timers.target
```

## Installation

```bash
# Copy to user systemd directory
cp todo-queue.service ~/.config/systemd/user/
cp todo-queue.timer ~/.config/systemd/user/

# Enable and start
systemctl --user daemon-reload
systemctl --user enable --now todo-queue.timer
```

---

# Database Schema

## Tasks Table

```sql
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    priority TEXT NOT NULL,
    created_at TEXT NOT NULL,
    due_at TEXT,
    completed_at TEXT,
    tags TEXT,
    project TEXT,
    estimated_minutes INTEGER
);
```

## Config Table

```sql
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

---

# Priority Sorting Logic

```
Priority Weight:
┌───────────┬─────────┐
│ Critical  │    4    │
│ High      │    3    │
│ Medium    │    2    │
│ Low       │    1    │
└───────────┴─────────┘

Sort Order:
1. Priority (DESC) - Higher first
2. Due Date (ASC) - Earlier first
3. Created At (ASC) - Older first

SQL:
ORDER BY priority DESC, due_at ASC, created_at ASC
```

---

# Time Format Support

## Relative Time

| Input | Meaning |
|-------|---------|
| `2h` | 2 hours from now |
| `1d` | 1 day from now |
| `1w` | 1 week from now |

## Absolute Time

| Input | Format |
|-------|--------|
| `2024-12-31 23:59` | Custom |
| `2024-12-31` | Date only |
| `14:30` | Time only |
| RFC3339 | ISO 8601 |

---

# Error Handling Strategy

```rust
use anyhow::{Context, Result};

// Provide context for errors
fn parse_time(s: &str) -> Result<DateTime<Utc>> {
    let hours: i64 = s.parse()
        .context("Failed to parse hours")?;
    Ok(Utc::now() + Duration::hours(hours))
}

// Graceful degradation
if config.use_notify_send {
    let _ = Command::new("notify-send")
        .args([...])
        .status();  // Ignore errors
}
```

---

# Future Enhancements

## Planned Features

- [ ] Interactive TUI mode (ratatui)
- [ ] Task dependencies
- [ ] Recurring tasks
- [ ] Task templates
- [ ] Export/Import (JSON, CSV)
- [ ] Sync with external services
- [ ] Web interface

## Potential Dependencies

- `ratatui`: Already included, ready for TUI
- `reqwest`: For API integrations
- `tokio`: Async support
- `config`: Configuration file support

---

# Summary

## Key Takeaways

1. **Modular Architecture**: Clean separation of concerns
2. **Type Safety**: Rust's type system prevents bugs
3. **Persistent Storage**: SQLite for reliability
4. **Extensible Design**: Easy to add new features
5. **Unix Philosophy**: Do one thing well

## Technology Stack

- **Language**: Rust
- **Database**: SQLite (rusqlite)
- **CLI**: clap
- **UI**: colored, crossterm, ratatui
- **Time**: chrono
- **Errors**: anyhow

---

# Thank You!

Questions?