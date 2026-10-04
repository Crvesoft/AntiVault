use rusqlite::params;
use crate::error::AppError;
use crate::storage::account::Database;

impl Database {
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get(0));
        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::DatabaseError(e.to_string())),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = self.conn()?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now],
        )?;
        Ok(())
    }
}
