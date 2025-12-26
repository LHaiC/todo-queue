# Todo Queue

An intelligent CLI task management system built with Rust, featuring priority-based queuing, persistent SQLite storage, and automated reminders.

## Features

- **Priority Queue**: Tasks sorted by priority (Critical > High > Medium > Low) and due date
- **Persistent Storage**: SQLite database for reliable data persistence
- **Smart Reminders**: Integrated with systemd timers for automated notifications
- **Rich Metadata**: Support for projects, tags, descriptions, and time estimates
- **Flexible Time Input**: Relative time (`2h`, `1d`, `1w`) and absolute time (`2024-12-31 23:59`)
- **Task Management**: Add, update, complete, delete, and view tasks
- **Statistics**: Track task completion and overdue status

## Installation

```bash
# Build the project
cargo build --release

# Run the installation script
./install.sh

# Enable reminder service
systemctl --user enable --now todo-queue.timer
```

## Usage

```bash
# Add a task with priority and tags
todo add "Complete documentation" --priority high --tags "work,docs" --estimate 60

# Add a task with due time (relative)
todo add "Fix critical bug" --priority critical --due "2h"

# Add a task with due time (absolute)
todo add "Submit report" --due "2024-12-31 23:59" --project "Project X"

# List pending tasks
todo list

# List all tasks (including completed)
todo list --completed

# List tasks by project
todo list --project "Project X"

# View next task
todo next

# Complete task (next or by ID)
todo done
todo done 3

# Update a task
todo update 1 --title "New title"
todo update 1 --priority high
todo update 1 --due "1d"
todo update 1 --title "Updated" --priority critical --due "1d"

# Show task details
todo show 5

# Delete a task
todo delete 2

# Clear all completed tasks
todo clear

# View statistics
todo stats

# Check reminders manually
todo remind
```

## Commands

| Command | Description | Options |
|---------|-------------|---------|
| `add` | Add a new task | `--title`, `--description`, `--priority`, `--due`, `--project`, `--tags`, `--estimate` |
| `list` | List tasks | `--completed`, `--project` |
| `next` | Show next task | - |
| `done` | Complete a task | `[ID]` |
| `update` | Update a task | `[ID]`, `--title`, `--description`, `--priority`, `--due`, `--project`, `--tags`, `--estimate` |
| `show` | Show task details | `[ID]` |
| `delete` | Delete a task | `[ID]` |
| `clear` | Clear completed tasks | - |
| `stats` | Show statistics | - |
| `remind` | Check reminders | - |

## Priority Levels

- 🔴 **Critical**: Urgent tasks (highest priority)
- 🟠 **High**: Important tasks
- 🟡 **Medium**: Normal tasks (default)
- 🟢 **Low**: Less important tasks

## Data Storage

All data is stored in `~/.todo-queue/tasks.db` (SQLite database).

## Systemd Integration

The project includes systemd service files for automated reminders:

- `todo-queue.service`: Executes the reminder check
- `todo-queue.timer`: Triggers the service every 30 minutes

## Project Structure

```
todo-queue/
├── src/
│   ├── main.rs       # CLI entry point and command routing
│   ├── models.rs     # Data models (Task, Priority, ReminderConfig)
│   ├── database.rs   # SQLite database operations
│   ├── ui.rs         # Terminal UI formatting and display
│   └── reminders.rs  # Reminder notification logic
├── Cargo.toml        # Project dependencies
├── install.sh        # Installation script
└── todo-queue.*      # Systemd service files
```

## Documentation

- [README.md](README.md) - This file
- [ARCHITECTURE.md](ARCHITECTURE.md) - Architecture documentation with Marp slides
- [TUTORIAL.md](TUTORIAL.md) - Comprehensive Rust tutorial for beginners

## License

MIT
