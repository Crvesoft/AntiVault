use std::time::Duration;
use chrono::DateTime;
use serde::Deserialize;
use serde_json::json;
use crate::utils::http::build_local_http_client;

#[derive(Debug, Clone)]
pub struct LocalQuotaResult {
    pub summary_items: Vec<(String, Option<f64>, Option<i64>)>,
    pub model_items: Vec<(String, Option<f64>, Option<i64>)>,
    pub tier: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LsQuotaSummaryEnvelope {
    response: Option<LsQuotaSummaryResponse>,
}

#[derive(Debug, Deserialize)]
struct LsQuotaSummaryResponse {
    groups: Option<Vec<LsQuotaSummaryGroup>>,
}

#[derive(Debug, Deserialize)]
struct LsQuotaSummaryGroup {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    buckets: Option<Vec<LsQuotaSummaryBucket>>,
}

#[derive(Debug, Deserialize)]
struct LsQuotaSummaryBucket {
    #[serde(rename = "bucketId")]
    bucket_id: Option<String>,
    window: Option<String>,
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LsUserStatusEnvelope {
    #[serde(rename = "userStatus")]
    user_status: Option<LsUserStatusData>,
}

#[derive(Debug, Deserialize)]
struct LsUserStatusData {
    #[serde(rename = "cascadeModelConfigData")]
    cascade_model_config_data: Option<LsCascadeModelConfigData>,
    #[serde(rename = "userTier")]
    user_tier: Option<LsUserTier>,
}

#[derive(Debug, Deserialize)]
struct LsCascadeModelConfigData {
    #[serde(rename = "clientModelConfigs")]
    client_model_configs: Option<Vec<LsClientModelConfig>>,
}

#[derive(Debug, Deserialize)]
struct LsClientModelConfig {
    label: Option<String>,
    #[serde(rename = "quotaInfo")]
    quota_info: Option<LsModelQuotaInfo>,
}

#[derive(Debug, Deserialize)]
struct LsModelQuotaInfo {
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LsUserTier {
    name: Option<String>,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct MibTcpRowOwnerPid {
    dw_state: u32,
    dw_local_addr: u32,
    dw_local_port: u32,
    dw_remote_addr: u32,
    dw_remote_port: u32,
    dw_owning_pid: u32,
}

#[cfg(target_os = "windows")]
#[link(name = "iphlpapi")]
extern "system" {
    fn GetExtendedTcpTable(
        p_tcp_table: *mut std::ffi::c_void,
        pdw_size: *mut u32,
        b_order: i32,
        ul_af: u32,
        table_class: i32,
        reserved: u32,
    ) -> u32;
}

#[cfg(target_os = "windows")]
pub fn get_listening_tcp_ports_for_pid(target_pid: u32) -> Vec<u16> {
    let mut size = 0u32;
    // AF_INET = 2, TCP_TABLE_OWNER_PID_LISTENER = 5
    unsafe {
        let _ = GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 0, 2, 5, 0);
        if size == 0 {
            return Vec::new();
        }
        let mut buf = vec![0u8; size as usize];
        if GetExtendedTcpTable(buf.as_mut_ptr() as *mut _, &mut size, 0, 2, 5, 0) != 0 {
            return Vec::new();
        }

        let num_entries = *(buf.as_ptr() as *const u32);
        let row_size = std::mem::size_of::<MibTcpRowOwnerPid>();
        let mut offset = 4;
        let mut ports = Vec::new();

        for _ in 0..num_entries {
            if offset + row_size > buf.len() {
                break;
            }
            let row = &*(buf.as_ptr().add(offset) as *const MibTcpRowOwnerPid);
            offset += row_size;

            if row.dw_owning_pid == target_pid {
                let port = u16::from_be((row.dw_local_port & 0xFFFF) as u16);
                ports.push(port);
            }
        }
        ports.sort_unstable();
        ports.dedup();
        ports
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_listening_tcp_ports_for_pid(_target_pid: u32) -> Vec<u16> {
    Vec::new()
}

fn extract_csrf_token(cmd: &str) -> Option<String> {
    if let Some(pos) = cmd.find("--csrf_token") {
        let after = &cmd[pos + "--csrf_token".len()..];
        let after = after.trim_start_matches(|c| c == '=' || c == ' ');
        let token: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '-').collect();
        if !token.is_empty() {
            return Some(token);
        }
    }
    None
}

pub fn find_all_antigravity_ls_candidates() -> Vec<(u32, String, Vec<u16>)> {
    let mut system = sysinfo::System::new();
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        sysinfo::ProcessRefreshKind::nothing().with_cmd(sysinfo::UpdateKind::Always),
    );

    let mut candidates = Vec::new();
    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        let cmd = process
            .cmd()
            .iter()
            .map(|a| a.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");

        if name.contains("language_server") || cmd.contains("--csrf_token") {
            if let Some(csrf) = extract_csrf_token(&cmd) {
                let pid_u32 = pid.as_u32();
                let ports = get_listening_tcp_ports_for_pid(pid_u32);
                if !ports.is_empty() {
                    candidates.push((pid_u32, csrf, ports));
                }
            }
        }
    }
    candidates
}

pub async fn fetch_local_ls_quota() -> Option<LocalQuotaResult> {
    let candidates = find_all_antigravity_ls_candidates();
    if candidates.is_empty() {
        return None;
    }
    let client = build_local_http_client(Duration::from_millis(1500)).ok()?;

    for (pid, csrf, ports) in candidates {
        for port in ports {
            for proto in &["https", "http"] {
                let summary_url = format!("{}://127.0.0.1:{}/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary", proto, port);
                let resp = client
                    .post(&summary_url)
                    .header("Content-Type", "application/json")
                    .header("X-Codeium-Csrf-Token", &csrf)
                    .header("Connect-Protocol-Version", "1")
                    .json(&json!({}))
                    .send()
                    .await;

                if let Ok(res) = resp {
                    if res.status().is_success() {
                        if let Ok(envelope) = res.json::<LsQuotaSummaryEnvelope>().await {
                            let mut summary_items = Vec::new();
                            if let Some(r) = envelope.response {
                                if let Some(groups) = r.groups {
                                    for group in groups {
                                        let g_name = group.display_name.unwrap_or_default().to_lowercase();
                                        let is_claude = g_name.contains("claude") || g_name.contains("3p") || g_name.contains("gpt");
                                        let is_gemini = g_name.contains("gemini") || g_name.contains("flash");

                                        if let Some(buckets) = group.buckets {
                                            for b in buckets {
                                                let window = b.window.unwrap_or_default().to_lowercase();
                                                let bid = b.bucket_id.unwrap_or_default().to_lowercase();
                                                let reset_at = b.reset_time.as_ref().and_then(|t| {
                                                    DateTime::parse_from_rfc3339(t).ok().map(|dt| dt.timestamp())
                                                });
                                                let rem = b.remaining_fraction;

                                                if is_gemini {
                                                    if window == "5h" || bid == "gemini-5h" {
                                                        summary_items.push(("Gemini (5h)".to_string(), rem, reset_at));
                                                    } else if window == "weekly" || bid == "gemini-weekly" {
                                                        summary_items.push(("Gemini (Weekly)".to_string(), rem, reset_at));
                                                    }
                                                } else if is_claude {
                                                    if window == "5h" || bid == "3p-5h" {
                                                        summary_items.push(("Claude (5h)".to_string(), rem, reset_at));
                                                    } else if window == "weekly" || bid == "3p-weekly" {
                                                        summary_items.push(("Claude (Weekly)".to_string(), rem, reset_at));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if !summary_items.is_empty() {
                                // Also fetch model details and tier from GetUserStatus
                                let status_url = format!("{}://127.0.0.1:{}/exa.language_server_pb.LanguageServerService/GetUserStatus", proto, port);
                                let mut model_items = Vec::new();
                                let mut tier_name = None;

                                if let Ok(status_resp) = client
                                    .post(&status_url)
                                    .header("Content-Type", "application/json")
                                    .header("X-Codeium-Csrf-Token", &csrf)
                                    .header("Connect-Protocol-Version", "1")
                                    .json(&json!({"metadata": {"ideName": "antigravity"}}))
                                    .send()
                                    .await
                                {
                                    if status_resp.status().is_success() {
                                        if let Ok(status_data) = status_resp.json::<LsUserStatusEnvelope>().await {
                                            if let Some(status) = status_data.user_status {
                                                tier_name = status.user_tier.and_then(|t| t.name);
                                                if let Some(cascade) = status.cascade_model_config_data {
                                                    if let Some(configs) = cascade.client_model_configs {
                                                        for cfg in configs {
                                                            if let Some(label) = cfg.label {
                                                                let rem = cfg.quota_info.as_ref().and_then(|q| q.remaining_fraction);
                                                                let reset_at = cfg.quota_info.as_ref().and_then(|q| {
                                                                    q.reset_time.as_ref().and_then(|t| {
                                                                        DateTime::parse_from_rfc3339(t).ok().map(|dt| dt.timestamp())
                                                                    })
                                                                });
                                                                model_items.push((label, rem, reset_at));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                tracing::info!("Successfully fetched real-time quota from local Antigravity Language Server PID {} on port {}", pid, port);
                                return Some(LocalQuotaResult {
                                    summary_items,
                                    model_items,
                                    tier: tier_name,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ls_quota_summary() {
        let json_str = r#"{
            "response": {
                "groups": [
                    {
                        "displayName": "Gemini",
                        "buckets": [
                            {
                                "bucketId": "gemini-weekly",
                                "remainingFraction": 0.96396655,
                                "resetTime": "2026-10-24T12:42:08Z",
                                "window": "weekly"
                            },
                            {
                                "bucketId": "gemini-5h",
                                "remainingFraction": 0.918405,
                                "resetTime": "2026-10-19T05:05:51Z",
                                "window": "5h"
                            }
                        ]
                    },
                    {
                        "displayName": "Claude & Other Models",
                        "buckets": [
                            {
                                "bucketId": "3p-weekly",
                                "remainingFraction": 0.85,
                                "resetTime": "2026-10-24T12:00:00Z",
                                "window": "weekly"
                            }
                        ]
                    }
                ]
            }
        }"#;

        let envelope: LsQuotaSummaryEnvelope = serde_json::from_str(json_str).expect("Failed to parse");
        let groups = envelope.response.unwrap().groups.unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].display_name.as_deref(), Some("Gemini"));
        assert_eq!(groups[0].buckets.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_parse_ls_user_status() {
        let json_str = r#"{
            "userStatus": {
                "userTier": {
                    "name": "Google AI Pro"
                },
                "cascadeModelConfigData": {
                    "clientModelConfigs": [
                        {
                            "label": "Gemini 2.5 Pro",
                            "quotaInfo": {
                                "remainingFraction": 0.95,
                                "resetTime": "2026-10-24T12:00:00Z"
                            }
                        }
                    ]
                }
            }
        }"#;

        let envelope: LsUserStatusEnvelope = serde_json::from_str(json_str).expect("Failed to parse");
        let status = envelope.user_status.unwrap();
        let configs = status.cascade_model_config_data.unwrap().client_model_configs.unwrap_or_default();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].label.as_deref(), Some("Gemini 2.5 Pro"));
    }

    #[tokio::test]
    async fn test_live_fetch_ls() {
        if let Some(res) = fetch_local_ls_quota().await {
            assert!(!res.summary_items.is_empty());
        }
    }
}
