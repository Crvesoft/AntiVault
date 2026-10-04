use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use rusqlite::params;

use crate::error::AppError;
use crate::storage::account::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub subscription_type: Option<String>,
    pub status: String,
    pub is_current: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_used_at: Option<i64>,
    pub last_quota_update_at: Option<i64>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAccountRequest {
    pub email: String,
    pub display_name: Option<String>,
    pub refresh_token: String,
    pub access_token: Option<String>,
    pub subscription_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchResult {
    pub success: bool,
    pub was_running: bool,
    pub restarted: bool,
    pub message: Option<String>,
}

#[tauri::command]
pub async fn list_accounts(db: State<'_, Database>) -> Result<Vec<AccountInfo>, AppError> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, email, display_name, avatar_url, subscription_type, status, is_current,
                created_at, updated_at, last_used_at, last_quota_update_at, sort_order
         FROM accounts ORDER BY sort_order ASC, created_at ASC"
    )?;

    let accounts = stmt.query_map([], |row| {
        Ok(AccountInfo {
            id: row.get(0)?,
            email: row.get(1)?,
            display_name: row.get(2)?,
            avatar_url: row.get(3)?,
            subscription_type: row.get(4)?,
            status: row.get(5)?,
            is_current: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            last_used_at: row.get(9)?,
            last_quota_update_at: row.get(10)?,
            sort_order: row.get(11).unwrap_or(0),
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(accounts)
}

#[tauri::command]
pub async fn get_account(id: String, db: State<'_, Database>) -> Result<AccountInfo, AppError> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, email, display_name, avatar_url, subscription_type, status, is_current,
                created_at, updated_at, last_used_at, last_quota_update_at, sort_order
         FROM accounts WHERE id = ?1"
    )?;

    stmt.query_row(params![id], |row| {
        Ok(AccountInfo {
            id: row.get(0)?,
            email: row.get(1)?,
            display_name: row.get(2)?,
            avatar_url: row.get(3)?,
            subscription_type: row.get(4)?,
            status: row.get(5)?,
            is_current: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            last_used_at: row.get(9)?,
            last_quota_update_at: row.get(10)?,
            sort_order: row.get(11).unwrap_or(0),
        })
    }).map_err(|_| AppError::AccountNotFound(id))
}

#[tauri::command]
pub async fn get_current_account(db: State<'_, Database>) -> Result<Option<AccountInfo>, AppError> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, email, display_name, avatar_url, subscription_type, status, is_current,
                created_at, updated_at, last_used_at, last_quota_update_at, sort_order
         FROM accounts WHERE is_current = 1 LIMIT 1"
    )?;

    let result = stmt.query_row([], |row| {
        Ok(AccountInfo {
            id: row.get(0)?,
            email: row.get(1)?,
            display_name: row.get(2)?,
            avatar_url: row.get(3)?,
            subscription_type: row.get(4)?,
            status: row.get(5)?,
            is_current: true,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            last_used_at: row.get(9)?,
            last_quota_update_at: row.get(10)?,
            sort_order: row.get(11).unwrap_or(0),
        })
    });

    match result {
        Ok(account) => Ok(Some(account)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

#[tauri::command]
pub async fn add_account(request: AddAccountRequest, db: State<'_, Database>) -> Result<AccountInfo, AppError> {
    let now = chrono::Utc::now().timestamp();

    // `accounts.email` is UNIQUE, so signing in (or re-importing) an email that is
    // already stored must update that row instead of failing the whole insert.
    let existing: Option<(String, String)> = {
        let conn = db.conn()?;
        conn.query_row(
            "SELECT id, token_ref FROM accounts WHERE email = ?1",
            params![request.email],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .ok()
    };

    let (id, token_ref, is_new) = match existing {
        Some((id, token_ref)) => (id, token_ref, false),
        None => {
            let id = Uuid::new_v4().to_string();
            let token_ref = format!("antivault/account/{}", id);
            (id, token_ref, true)
        }
    };

    // Store refresh token in secure storage
    let entry = keyring::Entry::new("antivault", &token_ref)
        .map_err(|e| AppError::SecureStorageError(format!("Failed to create keyring entry: {}", e)))?;
    entry.set_password(&request.refresh_token)
        .map_err(|e| AppError::SecureStorageError(format!("Failed to store token: {}", e)))?;

    // Store access token if provided
    if let Some(ref access_token) = request.access_token {
        let access_ref = format!("antivault/access/{}", id);
        let access_entry = keyring::Entry::new("antivault", &access_ref)
            .map_err(|e| AppError::SecureStorageError(format!("Failed to create access token entry: {}", e)))?;
        access_entry.set_password(access_token)
            .map_err(|e| AppError::SecureStorageError(format!("Failed to store access token: {}", e)))?;
    }

    {
        let conn = db.conn()?;

        if is_new {
            // Check if any accounts exist, if not, make this the current one
            let count: i32 = conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
            let is_current = if count == 0 { 1 } else { 0 };

            let next_order: i64 = conn.query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM accounts",
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            conn.execute(
                "INSERT INTO accounts (id, email, display_name, subscription_type, token_ref, status, is_current, created_at, updated_at, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'active', ?6, ?7, ?8, ?9)",
                params![id, request.email, request.display_name, request.subscription_type, token_ref, is_current, now, now, next_order],
            )?;
        } else {
            conn.execute(
                "UPDATE accounts SET
                    display_name = COALESCE(?1, display_name),
                    subscription_type = COALESCE(?2, subscription_type),
                    status = 'active',
                    updated_at = ?3
                 WHERE id = ?4",
                params![request.display_name, request.subscription_type, now, id],
            )?;
        }
    }

    get_account(id, db).await
}

#[tauri::command]
pub async fn delete_account(id: String, db: State<'_, Database>) -> Result<(), AppError> {
    let conn = db.conn()?;

    // Get token_ref to delete from secure storage
    let token_ref: String = conn.query_row(
        "SELECT token_ref FROM accounts WHERE id = ?1",
        params![id],
        |row| row.get(0),
    ).map_err(|_| AppError::AccountNotFound(id.clone()))?;

    // Delete from secure storage (best effort)
    if let Ok(entry) = keyring::Entry::new("antivault", &token_ref) {
        let _ = entry.delete_credential();
    }

    // Delete access token too
    let access_ref = format!("antivault/access/{}", id);
    if let Ok(entry) = keyring::Entry::new("antivault", &access_ref) {
        let _ = entry.delete_credential();
    }

    // Delete from database (cascades to quotas)
    conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])?;

    Ok(())
}

#[tauri::command]
pub async fn set_current_account(id: String, db: State<'_, Database>) -> Result<(), AppError> {
    let conn = db.conn()?;
    let now = chrono::Utc::now().timestamp();

    // Verify account exists
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = ?1)",
        params![id],
        |row| row.get(0),
    )?;

    if !exists {
        return Err(AppError::AccountNotFound(id));
    }

    // Unset all current
    conn.execute("UPDATE accounts SET is_current = 0, updated_at = ?1", params![now])?;

    // Set the target as current
    conn.execute(
        "UPDATE accounts SET is_current = 1, last_used_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;

    Ok(())
}

#[tauri::command]
pub async fn reorder_accounts(account_ids: Vec<String>, db: State<'_, Database>) -> Result<(), AppError> {
    let mut conn = db.conn()?;
    let tx = conn.transaction().map_err(|e| AppError::DatabaseError(e.to_string()))?;
    for (idx, id) in account_ids.iter().enumerate() {
        tx.execute(
            "UPDATE accounts SET sort_order = ?1 WHERE id = ?2",
            params![idx as i64, id],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    }
    tx.commit().map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

async fn fetch_google_userinfo_fast(access_token: &str) -> (Option<String>, Option<String>, Option<String>) {
    if access_token.is_empty() {
        return (None, None, None);
    }
    let client = crate::utils::http::build_http_client(std::time::Duration::from_secs(10))
        .unwrap_or_default();
    if let Ok(resp) = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "antigravity/1.10.0")
        .send()
        .await
    {
        if let Ok(info) = resp.json::<serde_json::Value>().await {
            let email = info.get("email").and_then(|v| v.as_str()).map(|s| s.to_string());
            let name = info.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
            let picture = info.get("picture").and_then(|v| v.as_str()).map(|s| s.to_string());
            return (email, name, picture);
        }
    }
    (None, None, None)
}

#[tauri::command]
pub async fn import_local_accounts(db: State<'_, Database>) -> Result<Vec<AccountInfo>, AppError> {
    let mut discovered: Vec<AccountInfo> = Vec::new();

    // 1. Scan Antigravity 2.0 (Windows Credential Manager gemini:antigravity)
    #[cfg(target_os = "windows")]
    {
        if let Some((refresh_token, access_token, _id_token)) = crate::utils::keyring_win::read_antigravity_20_credential() {
            let token_to_use = match access_token {
                Some(acc) if !acc.is_empty() => acc,
                _ => match crate::commands::quota::refresh_access_token_via_oauth(&refresh_token).await {
                    Ok((acc, _, _)) => acc,
                    Err(_) => String::new(),
                },
            };

            let (email_opt, name_opt, _) = fetch_google_userinfo_fast(&token_to_use).await;
            let email = email_opt.unwrap_or_else(|| "antigravity.v2@local".to_string());

            let exists: bool = {
                let conn = db.conn()?;
                conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM accounts WHERE email = ?1)",
                    params![email],
                    |row| row.get(0),
                )?
            };

            if !exists {
                let request = AddAccountRequest {
                    email: email.clone(),
                    display_name: name_opt.or_else(|| Some("Antigravity 2.0 账号".to_string())),
                    refresh_token,
                    access_token: if token_to_use.is_empty() { None } else { Some(token_to_use) },
                    subscription_type: None,
                };
                if let Ok(account) = add_account(request, db.clone()).await {
                    discovered.push(account);
                }
            }
        }
    }

    // 2. Try to find state.vscdb in known locations (IDE)
    let candidates = get_antigravity_db_candidates();

    for path in &candidates {
        if path.exists() {
            if let Ok(accounts) = scan_antigravity_db(path) {
                for (email_opt, refresh_token, access_token) in accounts {
                    let token_to_use = match access_token.as_ref() {
                        Some(acc) if !acc.is_empty() => acc.clone(),
                        _ => match crate::commands::quota::refresh_access_token_via_oauth(&refresh_token).await {
                            Ok((acc, _, _)) => acc,
                            Err(_) => String::new(),
                        },
                    };

                    let (info_email, info_name, _) = fetch_google_userinfo_fast(&token_to_use).await;
                    let email = email_opt
                        .filter(|e| !e.is_empty() && e != "Antigravity Account")
                        .or(info_email)
                        .unwrap_or_else(|| "antigravity.ide@local".to_string());

                    // Check if already imported
                    let exists: bool = {
                        let conn = db.conn()?;
                        conn.query_row(
                            "SELECT EXISTS(SELECT 1 FROM accounts WHERE email = ?1)",
                            params![email],
                            |row| row.get(0),
                        )?
                    };

                    if !exists {
                        let request = AddAccountRequest {
                            email: email.clone(),
                            display_name: info_name.or_else(|| Some("Antigravity IDE 账号".to_string())),
                            refresh_token,
                            access_token: if token_to_use.is_empty() { None } else { Some(token_to_use) },
                            subscription_type: None,
                        };

                        match add_account(request, db.clone()).await {
                            Ok(account) => discovered.push(account),
                            Err(e) => {
                                tracing::warn!("Failed to import account {}: {}", email, e);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(discovered)
}

fn get_antigravity_db_candidates() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        for folder in &["Antigravity IDE", "Antigravity"] {
            paths.push(
                std::path::PathBuf::from(&appdata)
                    .join(folder)
                    .join("User")
                    .join("globalStorage")
                    .join("state.vscdb"),
            );
        }
    }

    #[cfg(target_os = "macos")]
    if let Some(home) = dirs::home_dir() {
        for folder in &["Antigravity IDE", "Antigravity"] {
            paths.push(
                home.join("Library")
                    .join("Application Support")
                    .join(folder)
                    .join("User")
                    .join("globalStorage")
                    .join("state.vscdb"),
            );
        }
    }

    paths
}

pub(crate) fn scan_antigravity_db(path: &std::path::Path) -> Result<Vec<(Option<String>, String, Option<String>)>, AppError> {
    use rusqlite::Connection;

    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| AppError::DatabaseError(format!("Failed to open Antigravity DB: {}", e)))?;

    let mut accounts = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT key, value FROM ItemTable WHERE key = 'antigravityUnifiedStateSync.oauthToken'"
    ).map_err(|e| AppError::DatabaseError(format!("Failed to query ItemTable: {}", e)))?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
        ))
    }).map_err(|e| AppError::DatabaseError(format!("Failed to read rows: {}", e)))?;

    for row_result in rows {
        if let Ok((_key, val_str)) = row_result {
            if let Ok((refresh_token, access_token, email_opt)) = crate::utils::protobuf::extract_tokens_from_state_sync(&val_str) {
                accounts.push((email_opt, refresh_token, access_token));
            }
        }
    }

    Ok(accounts)
}

