use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use crate::error::AppError;

#[derive(Serialize, Deserialize)]
struct KeyringTokenDetails {
    access_token: String,
    token_type: String,
    refresh_token: String,
    expiry: String,
}

#[derive(Serialize, Deserialize)]
struct KeyringPayload {
    token: KeyringTokenDetails,
    auth_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id_token: Option<String>,
}

#[repr(C)]
struct FILETIME {
    dw_low_date_time: u32,
    dw_high_date_time: u32,
}

#[repr(C)]
struct CREDENTIALW {
    flags: u32,
    cred_type: u32,
    target_name: *const u16,
    comment: *const u16,
    last_written: FILETIME,
    credential_blob_size: u32,
    credential_blob: *const u8,
    persist: u32,
    attribute_count: u32,
    attributes: *const std::ffi::c_void,
    target_alias: *const u16,
    user_name: *const u16,
}

#[link(name = "advapi32")]
extern "system" {
    fn CredWriteW(credential: *const CREDENTIALW, flags: u32) -> i32;
    fn CredReadW(target_name: *const u16, cred_type: u32, flags: u32, credential: *mut *mut CREDENTIALW) -> i32;
    fn CredFree(buffer: *mut std::ffi::c_void);
}

/// Writes credentials to Windows Credential Manager under `gemini:antigravity`
/// exactly in the format required by Antigravity 2.0 and `agy` CLI
pub fn write_antigravity_20_credential(
    access_token: &str,
    refresh_token: &str,
    expiry_timestamp: i64,
    id_token: Option<&str>,
) -> Result<(), AppError> {
    let expiry_datetime = chrono::DateTime::from_timestamp(expiry_timestamp, 0)
        .unwrap_or_else(Utc::now);
    let expiry_str = expiry_datetime.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

    let payload = KeyringPayload {
        token: KeyringTokenDetails {
            access_token: access_token.to_string(),
            token_type: "Bearer".to_string(),
            refresh_token: refresh_token.to_string(),
            expiry: expiry_str,
        },
        auth_method: "consumer".to_string(),
        id_token: id_token.map(|s| s.to_string()),
    };

    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| AppError::Internal(format!("Failed to serialize Antigravity 2.0 payload: {}", e)))?;

    let target = "gemini:antigravity";
    let user = "antigravity";
    let secret = payload_json.as_bytes();

    let target_wide: Vec<u16> = OsStr::new(target).encode_wide().chain(std::iter::once(0)).collect();
    let user_wide: Vec<u16> = OsStr::new(user).encode_wide().chain(std::iter::once(0)).collect();

    let cred = CREDENTIALW {
        flags: 0,
        cred_type: 1, // CRED_TYPE_GENERIC
        target_name: target_wide.as_ptr(),
        comment: ptr::null(),
        last_written: FILETIME {
            dw_low_date_time: 0,
            dw_high_date_time: 0,
        },
        credential_blob_size: secret.len() as u32,
        credential_blob: secret.as_ptr(),
        persist: 2, // CRED_PERSIST_LOCAL_MACHINE
        attribute_count: 0,
        attributes: ptr::null(),
        target_alias: ptr::null(),
        user_name: user_wide.as_ptr(),
    };

    unsafe {
        let res = CredWriteW(&cred, 0);
        if res == 0 {
            let err = std::io::Error::last_os_error();
            return Err(AppError::SecureStorageError(format!("CredWriteW failed for gemini:antigravity: {}", err)));
        }
    }

    Ok(())
}

/// Reads the current credentials from `gemini:antigravity` (Antigravity 2.0 / CLI)
pub fn read_antigravity_20_credential() -> Option<(String, Option<String>, Option<String>)> {
    let target = "gemini:antigravity";
    let target_wide: Vec<u16> = OsStr::new(target).encode_wide().chain(std::iter::once(0)).collect();

    let mut pcred: *mut CREDENTIALW = ptr::null_mut();

    unsafe {
        let res = CredReadW(target_wide.as_ptr(), 1, 0, &mut pcred);
        if res == 0 || pcred.is_null() {
            return None;
        }

        let cred = &*pcred;
        let blob_slice = std::slice::from_raw_parts(cred.credential_blob, cred.credential_blob_size as usize);
        let text = String::from_utf8_lossy(blob_slice).to_string();

        CredFree(pcred as *mut std::ffi::c_void);

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
            let refresh_token = val.pointer("/token/refresh_token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())?;

            let access_token = val.pointer("/token/access_token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let id_token = val.get("id_token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            return Some((refresh_token, access_token, id_token));
        }

        // If stored as raw refresh token
        if text.trim().starts_with("1//") {
            return Some((text.trim().to_string(), None, None));
        }

        None
    }
}
