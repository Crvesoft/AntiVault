use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Token expired for account: {0}")]
    TokenExpired(String),

    #[error("Token invalid: {0}")]
    TokenInvalid(String),

    #[error("Quota fetch failed: {0}")]
    QuotaFetchFailed(String),

    #[error("Antigravity not found")]
    AntigravityNotFound,

    #[error("Antigravity is running")]
    AntigravityRunning,

    #[error("Account switch failed: {0}")]
    SwitchFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Secure storage error: {0}")]
    SecureStorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        let (code, message) = match self {
            AppError::AccountNotFound(m) => ("ACCOUNT_NOT_FOUND", m.as_str()),
            AppError::AuthenticationFailed(m) => ("AUTH_FAILED", m.as_str()),
            AppError::TokenExpired(m) => ("TOKEN_EXPIRED", m.as_str()),
            AppError::TokenInvalid(m) => ("TOKEN_INVALID", m.as_str()),
            AppError::QuotaFetchFailed(m) => ("QUOTA_FETCH_FAILED", m.as_str()),
            AppError::AntigravityNotFound => ("ANTIGRAVITY_NOT_FOUND", "Antigravity not found"),
            AppError::AntigravityRunning => ("ANTIGRAVITY_RUNNING", "Antigravity is running"),
            AppError::SwitchFailed(m) => ("SWITCH_FAILED", m.as_str()),
            AppError::DatabaseError(m) => ("DATABASE_ERROR", m.as_str()),
            AppError::SecureStorageError(m) => ("SECURE_STORAGE_ERROR", m.as_str()),
            AppError::NetworkError(m) => ("NETWORK_ERROR", m.as_str()),
            AppError::Internal(m) => ("INTERNAL_ERROR", m.as_str()),
        };
        state.serialize_field("code", code)?;
        state.serialize_field("message", message)?;
        state.end()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::NetworkError(e.to_string())
    }
}
