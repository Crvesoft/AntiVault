use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::AppError;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &PathBuf) -> Result<Self, AppError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::DatabaseError(format!("Failed to create database directory: {}", e))
            })?;
        }

        let conn = Connection::open(path).map_err(|e| {
            AppError::DatabaseError(format!("Failed to open database: {}", e))
        })?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
            .map_err(|e| AppError::DatabaseError(format!("Failed to set pragmas: {}", e)))?;

        let db = Database {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("Failed to acquire lock: {}", e))
        })?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                display_name TEXT,
                avatar_url TEXT,
                subscription_type TEXT,
                token_ref TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'unknown',
                is_current INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                last_used_at INTEGER,
                last_quota_update_at INTEGER,
                sort_order INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS quotas (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                model_name TEXT NOT NULL,
                remaining_percent REAL,
                remaining_value REAL,
                limit_value REAL,
                reset_at INTEGER,
                status TEXT NOT NULL DEFAULT 'unknown',
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );"
        ).map_err(|e| AppError::DatabaseError(format!("Migration failed: {}", e)))?;

        // Add sort_order column to existing accounts table if it was created without it
        let _ = conn.execute("ALTER TABLE accounts ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0", []);

        Ok(())
    }

    pub fn conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
        self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("Failed to acquire lock: {}", e))
        })
    }
}

pub fn get_database_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("AntiVault");
    path.push("antivault.db");
    path
}
