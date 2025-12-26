use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

use crate::models::{ReminderConfig, Task};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open database")?;

        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
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
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn add_task(&self, task: &Task) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tasks (title, description, priority, created_at, due_at, completed_at, tags, project, estimated_minutes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                task.title,
                task.description,
                serde_json::to_string(&task.priority)?,
                task.created_at.to_rfc3339(),
                task.due_at.map(|d| d.to_rfc3339()),
                task.completed_at.map(|d| d.to_rfc3339()),
                serde_json::to_string(&task.tags)?,
                task.project,
                task.estimated_minutes,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_task(&self, id: i64) -> Result<Option<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, priority, created_at, due_at, completed_at, tags, project, estimated_minutes
             FROM tasks WHERE id = ?1"
        )?;

        let task = stmt
            .query_row(params![id], |row| {
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    priority: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    due_at: row.get::<_, Option<String>>(5)?.map(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .unwrap()
                            .with_timezone(&Utc)
                    }),
                    completed_at: row.get::<_, Option<String>>(6)?.map(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .unwrap()
                            .with_timezone(&Utc)
                    }),
                    tags: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                    project: row.get(8)?,
                    estimated_minutes: row.get(9)?,
                })
            })
            .optional()?;

        Ok(task)
    }

    pub fn list_tasks(&self, include_completed: bool) -> Result<Vec<Task>> {
        let query = if include_completed {
            "SELECT id, title, description, priority, created_at, due_at, completed_at, tags, project, estimated_minutes
             FROM tasks ORDER BY priority DESC, created_at ASC"
        } else {
            "SELECT id, title, description, priority, created_at, due_at, completed_at, tags, project, estimated_minutes
             FROM tasks WHERE completed_at IS NULL ORDER BY priority DESC, created_at ASC"
        };

        let mut stmt = self.conn.prepare(query)?;
        let tasks = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap()
                    .with_timezone(&Utc),
                due_at: row.get::<_, Option<String>>(5)?.map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                completed_at: row.get::<_, Option<String>>(6)?.map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                tags: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                project: row.get(8)?,
                estimated_minutes: row.get(9)?,
            })
        })?;

        tasks.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_next_task(&self) -> Result<Option<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, priority, created_at, due_at, completed_at, tags, project, estimated_minutes
             FROM tasks WHERE completed_at IS NULL
             ORDER BY priority DESC, due_at ASC, created_at ASC LIMIT 1"
        )?;

        let task = stmt
            .query_row([], |row| {
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    priority: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    due_at: row.get::<_, Option<String>>(5)?.map(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .unwrap()
                            .with_timezone(&Utc)
                    }),
                    completed_at: row.get::<_, Option<String>>(6)?.map(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .unwrap()
                            .with_timezone(&Utc)
                    }),
                    tags: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                    project: row.get(8)?,
                    estimated_minutes: row.get(9)?,
                })
            })
            .optional()?;

        Ok(task)
    }

    pub fn complete_task(&self, id: i64) -> Result<bool> {
        let rows = self.conn.execute(
            "UPDATE tasks SET completed_at = ?1 WHERE id = ?2 AND completed_at IS NULL",
            params![Utc::now().to_rfc3339(), id],
        )?;
        Ok(rows > 0)
    }

    pub fn delete_task(&self, id: i64) -> Result<bool> {
        let rows = self
            .conn
            .execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    pub fn clear_completed(&self) -> Result<u64> {
        let rows = self
            .conn
            .execute("DELETE FROM tasks WHERE completed_at IS NOT NULL", [])?;
        Ok(rows as u64)
    }

    pub fn reset_all(&self) -> Result<u64> {
        let rows = self.conn.execute("DELETE FROM tasks", [])?;
        Ok(rows as u64)
    }

    pub fn update_task(&self, id: i64, task: &Task) -> Result<bool> {
        let rows = self.conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, priority = ?3, 
             due_at = ?4, tags = ?5, project = ?6, estimated_minutes = ?7 
             WHERE id = ?8",
            params![
                task.title,
                task.description,
                serde_json::to_string(&task.priority)?,
                task.due_at.map(|d| d.to_rfc3339()),
                serde_json::to_string(&task.tags)?,
                task.project,
                task.estimated_minutes,
                id,
            ],
        )?;
        Ok(rows > 0)
    }

    pub fn get_config(&self) -> Result<ReminderConfig> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM config WHERE key = 'reminder_config'")?;
        let config = stmt
            .query_row([], |row| {
                let value: String = row.get(0)?;
                serde_json::from_str(&value)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })
            .optional()?;

        Ok(config.unwrap_or_default())
    }

    #[allow(dead_code)]
    pub fn save_config(&self, config: &ReminderConfig) -> Result<()> {
        let value = serde_json::to_string(config)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES ('reminder_config', ?1)",
            params![value],
        )?;
        Ok(())
    }
}
