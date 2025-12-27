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

The installation script will:
- Create a symlink to the binary in `~/.local/bin/`
- Install systemd service and timer files
- Configure the timer based on your reminder settings
- Reload systemd daemon

**Updating Timer After Configuration Changes:**

```bash
# Re-run the installation script to update timer
./install.sh

# Or manually reload systemd
systemctl --user daemon-reload
systemctl --user restart todo-queue.timer
```

## Usage

```bash
# Add a task with priority and tags
todo add "Complete documentation" --priority high --tags "work,docs" --estimate 60

# Add a task with due time (relative)
todo add "Fix critical bug" --priority critical --due "2h"

# Add a task with due time (absolute)
todo add "Submit report" --due "2024-12-31 23:59" --project "Project X"

# Add a task with spaces in title (use quotes)
todo add "Rewrite Something in Rust"

# List pending tasks
todo list

# List all tasks (including completed)
todo list --all

# List completed tasks only
todo list --completed

# List tasks by project
todo list --project "Project X"

# View next task
todo next

# Complete task (next, by index, or by title)
todo done
todo done 3
todo done "Fix critical bug"

# Update a task
todo update 1 --title "New title"
todo update 1 --priority high
todo update 1 --due "1d"
todo update 1 --title "Updated" --priority critical --due "1d"
todo update "task name" --priority critical

# Show task details
todo show 5
todo show "task name"

# Delete a task
todo delete 2
todo delete "task name"

# Clear all completed tasks
todo clear

# Reset - delete all tasks (with confirmation)
todo reset

# Configure reminder settings
todo config --show
todo config --enabled true
todo config --interval 15
todo config --notify true
todo config --wall true

# View statistics
todo stats

# Check reminders manually
todo remind
```

## Commands

| Command | Description | Options |
|---------|-------------|---------|
| `add` | Add a new task | `title`, `--description`, `--priority`, `--due`, `--project`, `--tags`, `--estimate` |
| `list` | List tasks | `--completed`, `--all`, `--project` |
| `next` | Show next task | - |
| `done` | Complete a task | `[INDEX_OR_TITLE]` |
| `update` | Update a task | `[INDEX_OR_TITLE]`, `--title`, `--description`, `--priority`, `--due`, `--project`, `--tags`, `--estimate` |
| `show` | Show task details | `[INDEX_OR_TITLE]` |
| `delete` | Delete a task | `[INDEX_OR_TITLE]` |
| `clear` | Clear completed tasks | - |
| `reset` | Reset - delete all tasks | - |
| `config` | Configure reminder settings | `--show`, `--enabled`, `--interval`, `--notify`, `--wall`, `--wall-quiet-start`, `--wall-quiet-end` |
| `stats` | Show statistics | - |
| `remind` | Check reminders | - |

## Priority Levels

- 🔴 **Critical**: Urgent tasks (highest priority)
- 🟠 **High**: Important tasks
- 🟡 **Medium**: Normal tasks (default)
- 🟢 **Low**: Less important tasks

## Task Status

- ✅ **Completed**: Task is finished (shown with strikethrough title)
- 🔲 **Pending**: Task is not yet completed

## Reminder Configuration

The `todo config` command allows you to customize reminder settings:

```bash
# View current configuration
todo config --show

# Enable or disable reminders
todo config --enabled true
todo config --enabled false

# Set reminder interval (supports hours, minutes, or plain numbers)
todo config --interval 2h      # 2 hours
todo config --interval 30m     # 30 minutes
todo config --interval 90      # 90 minutes

# Enable or disable desktop notifications
todo config --notify true
todo config --notify false

# Enable or disable terminal wall messages
todo config --wall true
todo config --wall false

# Set wall quiet hours (no wall messages during this time)
todo config --wall-quiet-start 18  # 6 PM
todo config --wall-quiet-end 9    # 9 AM

# Start reminders from quiet-end time
todo config --start-from-quiet-end true
```

**Configuration Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--enabled` | Enable or disable reminders | `true` |
| `--interval` | Reminder interval (e.g., `2h`, `30m`, `60`) | `180` (3 hours) |
| `--notify` | Desktop notifications (notify-send) | `true` |
| `--wall` | Terminal broadcast messages (wall) | `false` |
| `--wall-quiet-start` | Wall quiet hours start (0-23) | `18` (6 PM) |
| `--wall-quiet-end` | Wall quiet hours end (0-23) | `9` (9 AM) |
| `--start-from-quiet-end` | Start reminders from quiet-end time | `false` |

**Reminder Methods:**

- **Desktop Notifications**: Sends desktop notifications using `notify-send` (requires `libnotify-bin`)
  - No quiet hours - always sends notifications when reminders are triggered
  
- **Terminal Wall Messages**: Broadcasts messages to all logged-in users via `wall` command
  - Respects quiet hours - no wall messages between 6 PM and 9 AM by default
  - Can be customized with `--wall-quiet-start` and `--wall-quiet-end`

**Quiet Hours Behavior:**

- Quiet hours apply only to **wall messages**, not desktop notifications
- Default quiet hours: 18:00 (6 PM) to 09:00 (9 AM)
- During quiet hours, wall messages are suppressed but desktop notifications still work
- Cross-day quiet hours are supported (e.g., 18:00 to 09:00 means 6 PM to 9 AM next day)

**Interval Format:**

- `2h` = 2 hours
- `30m` = 30 minutes
- `90` = 90 minutes (assumes minutes if no suffix)
- Display format automatically shows hours and minutes (e.g., `1h 30m` for 90 minutes)

**Updating Systemd Timer:**

After changing configuration, update the systemd timer:

```bash
# Re-run the installation script to update timer
./install.sh

# Apply changes
systemctl --user daemon-reload
systemctl --user restart todo-queue.timer
```

The installation script automatically generates the timer configuration based on your reminder settings.

## Data Storage

All data is stored in `~/.todo-queue/tasks.db` (SQLite database).

## Systemd Integration

The installation script automatically sets up systemd service files for automated reminders:

- `todo-queue.service`: Executes the reminder check (generated by install.sh)
- `todo-queue.timer`: Triggers the service based on configuration (generated by install.sh)

**Timer Configuration:**

The timer interval is controlled by the reminder configuration:
- Default: Every 3 hours (at 00:00, 03:00, 06:00, ...)
- Can be customized with `todo config --interval <time>`
- After changing interval, run `./install.sh` to update the timer

**Manual Timer Update:**

```bash
# Update timer based on current configuration
./install.sh

# Apply changes
systemctl --user daemon-reload
systemctl --user restart todo-queue.timer

# Check next trigger time
systemctl --user list-timers | grep todo-queue
```

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
├── install.sh        # Installation and timer setup script
└── README.md         # This file
```
