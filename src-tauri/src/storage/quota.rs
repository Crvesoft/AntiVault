use rusqlite::params;
use crate::error::AppError;
use crate::storage::account::Database;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuotaRecord {
    pub id: String,
    pub account_id: String,
    pub model_name: String,
    pub remaining_percent: Option<f64>,
    pub remaining_value: Option<f64>,
    pub limit_value: Option<f64>,
    pub reset_at: Option<i64>,
    pub status: String,
    pub updated_at: i64,
}

impl Database {
    pub fn upsert_quota(&self, quota: &QuotaRecord) -> Result<(), AppError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO quotas (id, account_id, model_name, remaining_percent, remaining_value, limit_value, reset_at, status, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                remaining_percent = excluded.remaining_percent,
                remaining_value = excluded.remaining_value,
                limit_value = excluded.limit_value,
                reset_at = excluded.reset_at,
                status = excluded.status,
                updated_at = excluded.updated_at",
            params![
                quota.id,
                quota.account_id,
                quota.model_name,
                quota.remaining_percent,
                quota.remaining_value,
                quota.limit_value,
                quota.reset_at,
                quota.status,
                quota.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_quotas_for_account(&self, account_id: &str) -> Result<Vec<QuotaRecord>, AppError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, model_name, remaining_percent, remaining_value, limit_value, reset_at, status, updated_at
             FROM quotas WHERE account_id = ?1 ORDER BY model_name"
        )?;

        let quotas = stmt.query_map(params![account_id], |row| {
            Ok(QuotaRecord {
                id: row.get(0)?,
                account_id: row.get(1)?,
                model_name: row.get(2)?,
                remaining_percent: row.get(3)?,
                remaining_value: row.get(4)?,
                limit_value: row.get(5)?,
                reset_at: row.get(6)?,
                status: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(quotas)
    }

    pub fn delete_quotas_for_account(&self, account_id: &str) -> Result<(), AppError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM quotas WHERE account_id = ?1", params![account_id])?;
        Ok(())
    }
}
