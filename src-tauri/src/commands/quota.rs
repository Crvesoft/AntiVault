use std::collections::HashMap;
use chrono::Utc;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;
use serde_json::json;
use tauri::State;

use crate::error::AppError;
use crate::storage::account::Database;
use crate::storage::quota::QuotaRecord;
use crate::commands::account::AccountInfo;

const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

const USER_AGENT_VAL: &str = "antigravity/1.10.0 (Windows NT 10.0; Win64; x64)";

const USER_QUOTA_SUMMARY_ENDPOINTS: [&str; 2] = [
    "https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary",
    "https://daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary",
];

const CLOUD_CODE_LOAD_PROJECT_ENDPOINTS: [&str; 2] = [
    "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist",
    "https://daily-cloudcode-pa.googleapis.com/v1internal:loadCodeAssist",
];

const QUOTA_API_ENDPOINTS: [&str; 2] = [
    "https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels",
    "https://daily-cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels",
];

#[derive(Debug, Deserialize)]
struct TokenRefreshResponse {
    access_token: String,
    expires_in: Option<i64>,
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoadProjectResponse {
    #[serde(rename = "cloudaicompanionProject")]
    project_id: Option<String>,
    #[serde(rename = "currentTier")]
    current_tier: Option<TierInfo>,
    #[serde(rename = "paidTier")]
    paid_tier: Option<TierInfo>,
    #[serde(rename = "allowedTiers")]
    allowed_tiers: Option<Vec<TierInfo>>,
}

#[derive(Debug, Deserialize)]
struct TierInfo {
    id: Option<String>,
    name: Option<String>,
    is_default: Option<bool>,
    #[serde(rename = "availableCredits")]
    available_credits: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct FetchModelsResponse {
    models: Option<HashMap<String, ModelInfo>>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "quotaInfo")]
    quota_info: Option<ModelQuotaInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelQuotaInfo {
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RetrieveUserQuotaSummaryResponse {
    groups: Option<Vec<QuotaSummaryGroup>>,
    #[allow(dead_code)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QuotaSummaryGroup {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[allow(dead_code)]
    description: Option<String>,
    buckets: Option<Vec<QuotaSummaryBucket>>,
}

#[derive(Debug, Deserialize)]
struct QuotaSummaryBucket {
    #[serde(rename = "bucketId")]
    bucket_id: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    window: Option<String>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
    #[allow(dead_code)]
    description: Option<String>,
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
}

pub async fn refresh_access_token_via_oauth(refresh_token: &str) -> Result<(String, i64, Option<String>), AppError> {
    let client = crate::utils::http::build_http_client(std::time::Duration::from_secs(30))?;
    let client_id = crate::utils::get_oauth_client_id();
    let client_secret = crate::utils::get_oauth_client_secret();

    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let resp = client
        .post(TOKEN_URL)
        .header(USER_AGENT, USER_AGENT_VAL)
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::AuthenticationFailed(format!("Token refresh request error: {}", e)))?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(AppError::AuthenticationFailed(format!("Token refresh failed: {}", err_text)));
    }

    let token_res: TokenRefreshResponse = resp
        .json()
        .await
        .map_err(|e| AppError::AuthenticationFailed(format!("Failed to parse token response: {}", e)))?;

    let now = Utc::now().timestamp();
    let expires_in = token_res.expires_in.unwrap_or(3600);
    let expiry = now + expires_in;

    Ok((token_res.access_token, expiry, token_res.id_token))
}

pub async fn fetch_account_tier_and_project(
    access_token: &str,
) -> (Option<String>, Option<String>, Option<f64>) {
    let client = match crate::utils::http::build_http_client(std::time::Duration::from_secs(15)) {
        Ok(c) => c,
        Err(_) => return (None, None, None),
    };

    let body = json!({
        "metadata": {
            "ideType": "ANTIGRAVITY",
            "ideVersion": "1.10.0"
        }
    });

    for endpoint in CLOUD_CODE_LOAD_PROJECT_ENDPOINTS {
        let res = client
            .post(endpoint)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, USER_AGENT_VAL)
            .json(&body)
            .send()
            .await;

        if let Ok(res) = res {
            if res.status().is_success() {
                if let Ok(data) = res.json::<LoadProjectResponse>().await {
                    let project_id = data.project_id;
                    let credits = data
                        .paid_tier
                        .as_ref()
                        .and_then(|t| t.available_credits)
                        .or_else(|| {
                            data.current_tier
                                .as_ref()
                                .and_then(|t| t.available_credits)
                        });

                    let raw_tier = data
                        .paid_tier
                        .as_ref()
                        .and_then(|t| t.id.clone().or_else(|| t.name.clone()))
                        .or_else(|| {
                            data.current_tier
                                .as_ref()
                                .and_then(|t| t.id.clone().or_else(|| t.name.clone()))
                        })
                        .or_else(|| {
                            data.allowed_tiers.as_ref().and_then(|allowed| {
                                allowed
                                    .iter()
                                    .find(|t| t.id.as_deref() == Some("free-tier") || t.is_default == Some(true))
                                    .and_then(|t| t.id.clone().or_else(|| t.name.clone()))
                            })
                        })
                        .unwrap_or_else(|| "free-tier".to_string());

                    let tier_name = if raw_tier.to_lowercase().contains("pro") {
                        "PRO".to_string()
                    } else if raw_tier.to_lowercase().contains("ultra") {
                        "ULTRA".to_string()
                    } else if raw_tier.to_lowercase().contains("enterprise") {
                        "ENTERPRISE".to_string()
                    } else {
                        "FREE".to_string()
                    };

                    return (Some(tier_name), project_id, credits);
                }
            }
        }
    }

    (None, None, None)
}

pub async fn fetch_user_quota_summary(
    access_token: &str,
    project_id: Option<&str>,
) -> Result<Vec<(String, Option<f64>, Option<i64>)>, AppError> {
    let client = crate::utils::http::build_http_client(std::time::Duration::from_secs(15))?;

    let mut body = json!({});
    if let Some(pid) = project_id {
        body = json!({ "project": pid });
    }

    for endpoint in USER_QUOTA_SUMMARY_ENDPOINTS {
        let res = client
            .post(endpoint)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, USER_AGENT_VAL)
            .json(&body)
            .send()
            .await;

        if let Ok(res) = res {
            if res.status().is_success() {
                if let Ok(data) = res.json::<RetrieveUserQuotaSummaryResponse>().await {
                    let mut summary_records = Vec::new();
                    if let Some(groups) = data.groups {
                        for group in groups {
                            let g_name = group.display_name.unwrap_or_default();
                            let is_claude = g_name.to_lowercase().contains("claude") || g_name.to_lowercase().contains("3p");
                            let is_gemini = g_name.to_lowercase().contains("gemini");

                            if let Some(buckets) = group.buckets {
                                for b in buckets {
                                    let window = b.window.unwrap_or_default().to_lowercase();
                                    let bid = b.bucket_id.unwrap_or_default().to_lowercase();
                                    let reset_at = b.reset_time.as_ref().and_then(|t| {
                                        chrono::DateTime::parse_from_rfc3339(t).ok().map(|dt| dt.timestamp())
                                    });
                                    let rem = b.remaining_fraction;

                                    if is_claude {
                                        if window == "5h" || bid == "3p-5h" {
                                            summary_records.push(("Claude (5h)".to_string(), rem, reset_at));
                                        } else if window == "weekly" || bid == "3p-weekly" {
                                            summary_records.push(("Claude (Weekly)".to_string(), rem, reset_at));
                                        }
                                    } else if is_gemini {
                                        if window == "5h" || bid == "gemini-5h" {
                                            summary_records.push(("Gemini (5h)".to_string(), rem, reset_at));
                                        } else if window == "weekly" || bid == "gemini-weekly" {
                                            summary_records.push(("Gemini (Weekly)".to_string(), rem, reset_at));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !summary_records.is_empty() {
                        return Ok(summary_records);
                    }
                }
            }
        }
    }

    Err(AppError::QuotaFetchFailed("Failed to fetch user quota summary".into()))
}

pub async fn fetch_account_quotas(
    access_token: &str,
    project_id: Option<&str>,
) -> Result<Vec<(String, Option<f64>, Option<i64>)>, AppError> {
    let client = crate::utils::http::build_http_client(std::time::Duration::from_secs(15))?;

    let mut body = json!({});
    if let Some(pid) = project_id {
        body = json!({ "project": pid });
    }

    for endpoint in QUOTA_API_ENDPOINTS {
        let res = client
            .post(endpoint)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, USER_AGENT_VAL)
            .json(&body)
            .send()
            .await;

        if let Ok(res) = res {
            if res.status().is_success() {
                if let Ok(data) = res.json::<FetchModelsResponse>().await {
                    let mut result = Vec::new();
                    if let Some(models) = data.models {
                        for (model_id, info) in models {
                            let display_name = info.display_name.unwrap_or(model_id);
                            let remaining = info.quota_info.as_ref().and_then(|q| q.remaining_fraction);
                            let reset_at = info.quota_info.as_ref().and_then(|q| {
                                q.reset_time.as_ref().and_then(|t| {
                                    chrono::DateTime::parse_from_rfc3339(t)
                                        .ok()
                                        .map(|dt| dt.timestamp())
                                    })
                            });
                            result.push((display_name, remaining, reset_at));
                        }
                    }
                    return Ok(result);
                }
            }
        }
    }

    Err(AppError::QuotaFetchFailed("Failed to fetch available models quota from endpoints".into()))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuotaRefreshOutcome {
    pub quotas: Vec<QuotaRecord>,
    /// Set when the token refreshed fine but the quota endpoints could not be
    /// reached, so the caller can tell the user the data is stale instead of
    /// silently reporting success.
    pub warning: Option<String>,
}

#[tauri::command]
pub async fn refresh_quota(
    account_id: String,
    db: State<'_, Database>,
    app: tauri::AppHandle,
) -> Result<QuotaRefreshOutcome, AppError> {
    // 1. Get account from DB
    let token_ref = {
        let conn = db.conn()?;
        conn.query_row(
            "SELECT token_ref FROM accounts WHERE id = ?1",
            rusqlite::params![account_id],
            |row| row.get::<_, String>(0),
        ).map_err(|_| AppError::AccountNotFound(account_id.clone()))?
    };

    // 2. Read refresh token from keyring
    let entry = keyring::Entry::new("antivault", &token_ref)
        .map_err(|e| AppError::SecureStorageError(e.to_string()))?;
    let refresh_token = entry.get_password()
        .map_err(|e| AppError::SecureStorageError(format!("Could not read refresh token: {}", e)))?;

    // 3. Refresh access token
    let (access_token, _expiry, _id_token) = refresh_access_token_via_oauth(&refresh_token).await?;

    // Also update cached access token in keyring
    let access_ref = format!("antivault/access/{}", account_id);
    if let Ok(access_entry) = keyring::Entry::new("antivault", &access_ref) {
        let _ = access_entry.set_password(&access_token);
    }

    // 4. Fetch tier, project ID, and available AI credits
    let (tier, project_id, credits) = fetch_account_tier_and_project(&access_token).await;

    let now = Utc::now().timestamp();

    // 5. Store available credits if present
    if let Some(c) = credits {
        let quota_rec = QuotaRecord {
            id: format!("{}_AI_Credits", account_id),
            account_id: account_id.clone(),
            model_name: "AI Credits".to_string(),
            remaining_percent: None,
            remaining_value: Some(c),
            limit_value: None,
            reset_at: None,
            status: "active".to_string(),
            updated_at: now,
        };
        let _ = db.upsert_quota(&quota_rec);
    }

    // 6. Fetch 5h & weekly user quota summary
    let mut has_summary = false;
    match fetch_user_quota_summary(&access_token, project_id.as_deref()).await {
        Ok(summary_items) => {
            for (bucket_name, remaining, reset_at) in &summary_items {
                let clean_name = bucket_name.replace([' ', '(', ')'], "_");
                let quota_rec = QuotaRecord {
                    id: format!("{}_{}", account_id, clean_name),
                    account_id: account_id.clone(),
                    model_name: bucket_name.clone(),
                    remaining_percent: remaining.map(|r| (r * 100.0).round()),
                    remaining_value: None,
                    limit_value: None,
                    reset_at: *reset_at,
                    status: "active".to_string(),
                    updated_at: now,
                };
                let _ = db.upsert_quota(&quota_rec);
            }
            has_summary = true;
        }
        Err(_) => {
            // Will fallback from model quotas below if needed
        }
    }

    // 7. Fetch individual model quotas
    let mut warning: Option<String> = None;
    match fetch_account_quotas(&access_token, project_id.as_deref()).await {
        Ok(quotas) => {
            let mut claude_fallback: Option<(Option<f64>, Option<i64>)> = None;
            let mut gemini_fallback: Option<(Option<f64>, Option<i64>)> = None;

            // Normalize model names by stripping difficulty tags (High, Medium, Low) and merge
            let mut merged_models: std::collections::HashMap<String, (Option<f64>, Option<i64>)> = std::collections::HashMap::new();

            for (model_name, remaining, reset_at) in &quotas {
                let clean_name = model_name
                    .replace(" (High)", "")
                    .replace(" (Medium)", "")
                    .replace(" (Low)", "")
                    .replace("(High)", "")
                    .replace("(Medium)", "")
                    .replace("(Low)", "")
                    .trim()
                    .to_string();

                merged_models
                    .entry(clean_name)
                    .and_modify(|(rem, r_at)| {
                        if let Some(new_rem) = remaining {
                            if let Some(old_rem) = *rem {
                                if *new_rem < old_rem {
                                    *rem = Some(*new_rem);
                                    *r_at = *reset_at;
                                }
                            } else {
                                *rem = Some(*new_rem);
                                *r_at = *reset_at;
                            }
                        }
                    })
                    .or_insert((*remaining, *reset_at));

                let m_lower = model_name.to_lowercase();
                if claude_fallback.is_none() && m_lower.contains("claude") {
                    claude_fallback = Some((*remaining, *reset_at));
                }
                if gemini_fallback.is_none() && m_lower.contains("gemini") {
                    gemini_fallback = Some((*remaining, *reset_at));
                }
            }

            for (model_name, (remaining, reset_at)) in merged_models {
                let clean_id_name = model_name.replace([' ', '(', ')', '.', '-'], "_");
                let quota_rec = QuotaRecord {
                    id: format!("{}_{}", account_id, clean_id_name),
                    account_id: account_id.clone(),
                    model_name,
                    remaining_percent: remaining.map(|r| (r * 100.0).round()),
                    remaining_value: None,
                    limit_value: None,
                    reset_at,
                    status: "active".to_string(),
                    updated_at: now,
                };
                let _ = db.upsert_quota(&quota_rec);
            }

            // Fallback 5h summary if summary endpoint didn't succeed
            if !has_summary {
                if let Some((rem, r_at)) = claude_fallback {
                    let _ = db.upsert_quota(&QuotaRecord {
                        id: format!("{}_Claude__5h_", account_id),
                        account_id: account_id.clone(),
                        model_name: "Claude (5h)".to_string(),
                        remaining_percent: rem.map(|r| (r * 100.0).round()),
                        remaining_value: None,
                        limit_value: None,
                        reset_at: r_at,
                        status: "active".to_string(),
                        updated_at: now,
                    });
                }
                if let Some((rem, r_at)) = gemini_fallback {
                    let _ = db.upsert_quota(&QuotaRecord {
                        id: format!("{}_Gemini__5h_", account_id),
                        account_id: account_id.clone(),
                        model_name: "Gemini (5h)".to_string(),
                        remaining_percent: rem.map(|r| (r * 100.0).round()),
                        remaining_value: None,
                        limit_value: None,
                        reset_at: r_at,
                        status: "active".to_string(),
                        updated_at: now,
                    });
                }
            }
        }
        Err(e) => {
            if !has_summary {
                warning = Some(e.to_string());
            }
        }
    }

    // 8. Update account metadata (tier & last_quota_update_at)
    {
        let conn = db.conn()?;
        if let Some(ref t) = tier {
            let _ = conn.execute(
                "UPDATE accounts SET subscription_type = ?1, last_quota_update_at = ?2, status = 'active', updated_at = ?2 WHERE id = ?3",
                rusqlite::params![t, now, account_id],
            );
        } else {
            let _ = conn.execute(
                "UPDATE accounts SET last_quota_update_at = ?1, status = 'active', updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, account_id],
            );
        }
    }

    crate::tray::update_tray_menu(&app);

    Ok(QuotaRefreshOutcome {
        quotas: db.get_quotas_for_account(&account_id)?,
        warning,
    })
}

#[tauri::command]
pub async fn get_quotas(
    account_id: String,
    db: State<'_, Database>,
) -> Result<Vec<QuotaRecord>, AppError> {
    db.get_quotas_for_account(&account_id)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RefreshFailure {
    pub account_id: String,
    pub email: String,
    pub error: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RefreshAllResult {
    pub accounts: Vec<AccountInfo>,
    pub succeeded: Vec<String>,
    pub failed: Vec<RefreshFailure>,
}

#[tauri::command]
pub async fn refresh_all_quotas(
    db: State<'_, Database>,
    app: tauri::AppHandle,
) -> Result<RefreshAllResult, AppError> {
    let ids = {
        let conn = db.conn()?;
        let mut stmt = conn.prepare("SELECT id FROM accounts")?;
        let res = stmt.query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<String>, _>>()?;
        res
    };

    let mut succeeded: Vec<String> = Vec::new();
    let mut failed: Vec<RefreshFailure> = Vec::new();

    for id in ids {
        match refresh_quota(id.clone(), db.clone(), app.clone()).await {
            Ok(_) => succeeded.push(id),
            Err(e) => {
                let email = {
                    let conn = db.conn()?;
                    conn.query_row(
                        "SELECT email FROM accounts WHERE id = ?1",
                        rusqlite::params![id],
                        |row| row.get::<_, String>(0),
                    )
                    .unwrap_or_else(|_| id.clone())
                };
                failed.push(RefreshFailure {
                    account_id: id,
                    email,
                    error: e.to_string(),
                });
            }
        }
    }

    crate::tray::update_tray_menu(&app);

    let accounts = crate::commands::account::list_accounts(db).await?;

    Ok(RefreshAllResult {
        accounts,
        succeeded,
        failed,
    })
}
