#!/usr/bin/env bash
set -e

INSTALL_DIR="$HOME/.local/bin"
BINARY="./target/release/todo_queue"
DB_PATH="$HOME/.todo-queue/tasks.db"
SERVICE_PATH="$HOME/.config/systemd/user/todo-queue.service"
TIMER_PATH="$HOME/.config/systemd/user/todo-queue.timer"

# Function to update timer based on configuration
update_timer() {
    echo "🔄 Updating systemd timer based on configuration..."
    
    # Check if database exists
    if [ ! -f "$DB_PATH" ]; then
        echo "⚠️  Database not found, using default timer configuration"
        return
    fi
    
    # Get configuration
    INTERVAL_MINUTES=$(sqlite3 "$DB_PATH" "SELECT json_extract(value, '$.interval_minutes') FROM config WHERE key = 'reminder_config';" 2>/dev/null || echo "180")
    START_FROM_QUIET_END=$(sqlite3 "$DB_PATH" "SELECT json_extract(value, '$.start_from_quiet_end') FROM config WHERE key = 'reminder_config';" 2>/dev/null || echo "0")
    WALL_QUIET_END=$(sqlite3 "$DB_PATH" "SELECT json_extract(value, '$.wall_quiet_end_hour') FROM config WHERE key = 'reminder_config';" 2>/dev/null || echo "9")
    
    if [ -z "$INTERVAL_MINUTES" ]; then
        INTERVAL_MINUTES=180
    fi
    
    # Calculate hours and minutes
    HOURS=$((INTERVAL_MINUTES / 60))
    MINS=$((INTERVAL_MINUTES % 60))
    
    # Generate OnCalendar expression
    if [ "$START_FROM_QUIET_END" = "1" ] && [ -n "$WALL_QUIET_END" ]; then
        # Start from quiet-end time
        if [ "$HOURS" -ge 24 ]; then
            ON_CALENDAR_SPEC="*-*-* ${WALL_QUIET_END}:00"
        elif [ "$HOURS" -gt 0 ] && [ "$MINS" -eq 0 ]; then
            ON_CALENDAR_SPEC="*-*-* ${WALL_QUIET_END}:00/${HOURS}:00"
        else
            ON_CALENDAR_SPEC="*-*-* ${WALL_QUIET_END}:00/${INTERVAL_MINUTES}:00"
        fi
    else
        # Fixed interval from 00:00
        if [ "$HOURS" -ge 24 ]; then
            ON_CALENDAR_SPEC="*-*-* 00:00"
        elif [ "$HOURS" -gt 0 ] && [ "$MINS" -eq 0 ]; then
            ON_CALENDAR_SPEC="*-*-* 00:00/${HOURS}:00"
        else
            ON_CALENDAR_SPEC="*-*-* 00:00/${INTERVAL_MINUTES}:00"
        fi
    fi
    
    # Create timer file
    mkdir -p ~/.config/systemd/user
    cat > "$TIMER_PATH" << TIMEREOF
[Unit]
Description=Todo Queue Reminder Timer
Documentation=man:systemd.timer(5)

[Timer]
# Remind every ${INTERVAL_MINUTES} minutes (${HOURS}h ${MINS}m)
OnCalendar=${ON_CALENDAR_SPEC}
Persistent=true

[Install]
WantedBy=timers.target
TIMEREOF
    
    echo "✅ Timer updated: OnCalendar=${ON_CALENDAR_SPEC}"
}

# Main installation
echo "🚀 Installing todo_queue..."

# Create symlink
mkdir -p "$INSTALL_DIR"
ln -sf "$(pwd)/$BINARY" "$INSTALL_DIR/todo"

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "⚠️  Please add the following to your ~/.bashrc or ~/.zshrc:"
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Then run: source ~/.bashrc (or source ~/.zshrc)"
fi

# Install systemd service
echo ""
echo "📦 Installing systemd timer service..."
mkdir -p ~/.config/systemd/user

# Create service file
cat > "$SERVICE_PATH" << SERVICEEOF
[Unit]
Description=Todo Queue Reminder Service
Documentation=man:systemd.service(5)

[Service]
Type=oneshot
ExecStart=$(pwd)/$BINARY remind
SERVICEEOF

# Update timer based on configuration
update_timer

# Reload systemd
systemctl --user daemon-reload 2>/dev/null || true

echo ""
echo "✅ Installation complete!"
echo ""
echo "Usage:"
echo "  todo add \"task\"              # Add a task"
echo "  todo list                    # List tasks"
echo "  todo next                    # Show next task"
echo "  todo done                    # Complete task"
echo "  todo config --show           # Show configuration"
echo "  todo --help                  # Show help"
echo ""
echo "Enable reminders:"
echo "  systemctl --user enable --now todo-queue.timer"
echo ""
echo "Update timer after configuration changes:"
echo "  ./install.sh --update-timer"
echo ""
echo "Or manually:"
echo "  systemctl --user daemon-reload"
echo "  systemctl --user restart todo-queue.timer"