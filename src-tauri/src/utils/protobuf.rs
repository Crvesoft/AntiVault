use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::json;

use crate::error::AppError;

pub fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    while value >= 0x80 {
        buf.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }
    buf.push(value as u8);
    buf
}

pub fn read_varint(data: &[u8], offset: usize) -> Result<(u64, usize), AppError> {
    let mut result = 0u64;
    let mut shift = 0;
    let mut pos = offset;

    loop {
        if pos >= data.len() {
            return Err(AppError::Internal("Incomplete protobuf varint".into()));
        }
        let byte = data[pos];
        result |= ((byte & 0x7F) as u64) << shift;
        pos += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    Ok((result, pos))
}

pub fn encode_len_delim_field(field_num: u32, data: &[u8]) -> Vec<u8> {
    let tag = (field_num << 3) | 2;
    let mut f = encode_varint(tag as u64);
    f.extend(encode_varint(data.len() as u64));
    f.extend_from_slice(data);
    f
}

pub fn encode_string_field(field_num: u32, value: &str) -> Vec<u8> {
    encode_len_delim_field(field_num, value.as_bytes())
}

pub fn encode_varint_field(field_num: u32, value: u64) -> Vec<u8> {
    let tag = field_num << 3;
    let mut f = encode_varint(tag as u64);
    f.extend(encode_varint(value));
    f
}

pub fn create_oauth_info(
    access_token: &str,
    refresh_token: &str,
    expiry: i64,
    is_gcp_tos: bool,
    id_token: Option<&str>,
) -> Vec<u8> {
    let mut buf = Vec::new();
    // Field 1: access_token
    buf.extend(encode_string_field(1, access_token));
    // Field 2: token_type
    buf.extend(encode_string_field(2, "Bearer"));
    // Field 3: refresh_token
    buf.extend(encode_string_field(3, refresh_token));
    // Field 4: expiry timestamp as google.protobuf.Timestamp message { int64 seconds = 1; }
    let mut ts_buf = Vec::new();
    ts_buf.extend(encode_varint_field(1, expiry as u64));
    buf.extend(encode_len_delim_field(4, &ts_buf));
    // Field 5: id_token
    if let Some(id_tok) = id_token {
        if !id_tok.is_empty() {
            buf.extend(encode_string_field(5, id_tok));
        }
    }
    // Field 6: is_gcp_tos
    if is_gcp_tos {
        buf.extend(encode_varint_field(6, 1));
    }
    buf
}

/// Helper to scan for token with specific prefix
fn find_token_by_prefix(text: &str, prefix: &str) -> Option<String> {
    if let Some(pos) = text.find(prefix) {
        let remainder = &text[pos..];
        let token: String = remainder
            .chars()
            .take_while(|&c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '/' || c == '.')
            .collect();
        if token.len() >= prefix.len() + 10 {
            return Some(token);
        }
    }
    None
}

/// Constructs the full base64 string for `antigravityUnifiedStateSync.oauthToken` in `ItemTable`
pub fn build_state_sync_payload(
    access_token: &str,
    refresh_token: &str,
    expiry: i64,
    id_token: Option<&str>,
    is_gcp_tos: bool,
    email: &str,
    display_name: Option<&str>,
    avatar_url: Option<&str>,
) -> String {
    // 1. Inner protobuf OAuthTokenInfo
    let inner_pb = create_oauth_info(access_token, refresh_token, expiry, is_gcp_tos, id_token);
    let inner_b64 = BASE64.encode(&inner_pb);

    // 2. Message 1: oauthTokenInfoSentinelKey
    // Field 1: string "oauthTokenInfoSentinelKey"
    // Field 2: nested message with Field 1: inner_b64
    let mut msg1_inner = Vec::new();
    msg1_inner.extend(encode_string_field(1, &inner_b64));

    let mut msg1 = Vec::new();
    msg1.extend(encode_string_field(1, "oauthTokenInfoSentinelKey"));
    msg1.extend(encode_len_delim_field(2, &msg1_inner));

    // 3. Message 2: authStateWithContextSentinelKey
    let ctx_json = json!({
        "state": "signedIn",
        "context": {
            "project": "",
            "showProjectSelector": false,
            "email": email,
            "name": display_name.unwrap_or(email),
            "picture": avatar_url.unwrap_or("")
        }
    }).to_string();

    let mut msg2_inner = Vec::new();
    msg2_inner.extend(encode_string_field(1, &ctx_json));

    let mut msg2 = Vec::new();
    msg2.extend(encode_string_field(1, "authStateWithContextSentinelKey"));
    msg2.extend(encode_len_delim_field(2, &msg2_inner));

    // 4. Outer wrapper: Field 1 (msg1) + Field 1 (msg2)
    let mut outer = Vec::new();
    outer.extend(encode_len_delim_field(1, &msg1));
    outer.extend(encode_len_delim_field(1, &msg2));

    BASE64.encode(&outer)
}

/// Parses the base64 string from `antigravityUnifiedStateSync.oauthToken` to extract refresh_token, access_token, and email
pub fn extract_tokens_from_state_sync(

    base64_payload: &str,
) -> Result<(String, Option<String>, Option<String>), AppError> {
    let raw = BASE64.decode(base64_payload.trim())
        .map_err(|e| AppError::Internal(format!("Failed to decode base64 oauthToken: {}", e)))?;

    let mut email: Option<String> = None;
    let text = String::from_utf8_lossy(&raw);

    // Look for JSON context to find email
    if let Some(pos) = text.find("{\"state\":\"signedIn\"") {
        if let Some(end_pos) = text[pos..].find('}') {
            let json_slice = &text[pos..pos + end_pos + 1];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_slice) {
                if let Some(em) = val.pointer("/context/email").and_then(|v| v.as_str()) {
                    if !em.is_empty() {
                        email = Some(em.to_string());
                    }
                }
            }
        }
    }

    // Direct search in raw text
    let mut refresh_token = find_token_by_prefix(&text, "1//");
    let mut access_token = find_token_by_prefix(&text, "ya29.");

    // If not found directly, look inside any embedded base64 chunks
    if refresh_token.is_none() || access_token.is_none() {
        // Collect potential base64 substrings (alphanumeric + '+' + '/' + '=') with length >= 60
        let mut start_idx = None;
        let bytes = text.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            let is_b64 = b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=';
            if is_b64 && start_idx.is_none() {
                start_idx = Some(i);
            } else if !is_b64 && start_idx.is_some() {
                let start = start_idx.take().unwrap();
                if i - start >= 60 {
                    if let Ok(decoded) = BASE64.decode(&bytes[start..i]) {
                        let sub_text = String::from_utf8_lossy(&decoded);
                        if refresh_token.is_none() {
                            refresh_token = find_token_by_prefix(&sub_text, "1//");
                        }
                        if access_token.is_none() {
                            access_token = find_token_by_prefix(&sub_text, "ya29.");
                        }
                    }
                }
            }
        }
        if let Some(start) = start_idx {
            if bytes.len() - start >= 60 {
                if let Ok(decoded) = BASE64.decode(&bytes[start..]) {
                    let sub_text = String::from_utf8_lossy(&decoded);
                    if refresh_token.is_none() {
                        refresh_token = find_token_by_prefix(&sub_text, "1//");
                    }
                    if access_token.is_none() {
                        access_token = find_token_by_prefix(&sub_text, "ya29.");
                    }
                }
            }
        }
    }

    let refresh = refresh_token.ok_or_else(|| AppError::Internal("No refresh token found in state".into()))?;
    Ok((refresh, access_token, email))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload() {
        let p = build_state_sync_payload(
            "ya29.test",
            "1//test",
            1790000000,
            None,
            false,
            "test@gmail.com",
            Some("Test User"),
            None,
        );
        println!("Generated payload: {}", p);
    }
}

