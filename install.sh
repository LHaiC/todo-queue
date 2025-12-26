#!/usr/bin/env bash
set -e

INSTALL_DIR="$HOME/.local/bin"
BINARY="./target/release/todo_queue"

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
cp todo-queue.service ~/.config/systemd/user/
cp todo-queue.timer ~/.config/systemd/user/

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
echo "  todo --help                  # Show help"
echo ""
echo "Enable reminders:"
echo "  systemctl --user enable --now todo-queue.timer"