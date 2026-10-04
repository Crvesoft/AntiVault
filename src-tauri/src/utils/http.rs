use std::time::Duration;
use reqwest::{Client, ClientBuilder, Proxy};
use crate::error::AppError;

/// Detects the system proxy on Windows from registry, or from environment variables.
pub fn detect_system_proxy() -> Option<String> {
    // 1. Environment variables
    for var in &["HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy", "HTTP_PROXY", "http_proxy"] {
        if let Ok(val) = std::env::var(var) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // 2. Windows Internet Settings Registry
    #[cfg(target_os = "windows")]
    {
        if let Some(proxy) = read_windows_internet_settings_proxy() {
            return Some(proxy);
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn read_windows_internet_settings_proxy() -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    const HKEY_CURRENT_USER: isize = -2147483647isize; // 0x80000001
    const KEY_READ: u32 = 0x20019;
    const REG_DWORD: u32 = 4;
    const REG_SZ: u32 = 1;

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            h_key: isize,
            lp_sub_key: *const u16,
            ul_options: u32,
            sam_desired: u32,
            phk_result: *mut isize,
        ) -> i32;
        fn RegQueryValueExW(
            h_key: isize,
            lp_value_name: *const u16,
            lp_reserved: *const u32,
            lp_type: *mut u32,
            lp_data: *mut u8,
            lpcb_data: *mut u32,
        ) -> i32;
        fn RegCloseKey(h_key: isize) -> i32;
    }

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(Some(0)).collect()
    }

    let subkey = to_wide("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings");
    let mut hkey: isize = 0;
    unsafe {
        let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey);
        if status != 0 {
            return None;
        }

        // Check ProxyEnable
        let enable_name = to_wide("ProxyEnable");
        let mut val_type = 0u32;
        let mut enabled = 0u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        let status = RegQueryValueExW(
            hkey,
            enable_name.as_ptr(),
            std::ptr::null(),
            &mut val_type,
            &mut enabled as *mut u32 as *mut u8,
            &mut size,
        );

        if status != 0 || val_type != REG_DWORD || enabled == 0 {
            RegCloseKey(hkey);
            return None;
        }

        // Read ProxyServer
        let server_name = to_wide("ProxyServer");
        let mut server_size = 0u32;
        let status = RegQueryValueExW(
            hkey,
            server_name.as_ptr(),
            std::ptr::null(),
            &mut val_type,
            std::ptr::null_mut(),
            &mut server_size,
        );

        if status != 0 || val_type != REG_SZ || server_size == 0 {
            RegCloseKey(hkey);
            return None;
        }

        let mut buf = vec![0u8; server_size as usize];
        let status = RegQueryValueExW(
            hkey,
            server_name.as_ptr(),
            std::ptr::null(),
            &mut val_type,
            buf.as_mut_ptr(),
            &mut server_size,
        );
        RegCloseKey(hkey);

        if status != 0 {
            return None;
        }

        let u16_slice = std::slice::from_raw_parts(buf.as_ptr() as *const u16, (server_size as usize) / 2);
        let s = String::from_utf16_lossy(u16_slice);
        let raw = s.trim_end_matches('\0').trim();
        if raw.is_empty() {
            return None;
        }

        Some(normalize_proxy_spec(raw))
    }
}

/// Normalizes proxy specifications from Windows registry.
/// Examples:
/// - "127.0.0.1:10808" -> "http://127.0.0.1:10808"
/// - "http=127.0.0.1:10808;https=127.0.0.1:10808" -> "http://127.0.0.1:10808"
/// - "socks=127.0.0.1:10808" -> "socks5://127.0.0.1:10808"
pub fn normalize_proxy_spec(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.contains(';') || trimmed.contains('=') {
        let mut https_proxy = None;
        let mut http_proxy = None;
        let mut socks_proxy = None;

        for part in trimmed.split(';') {
            let part = part.trim();
            if let Some((proto, addr)) = part.split_once('=') {
                let addr = addr.trim();
                match proto.trim().to_lowercase().as_str() {
                    "https" => https_proxy = Some(addr),
                    "http" => http_proxy = Some(addr),
                    "socks" => socks_proxy = Some(addr),
                    _ => {}
                }
            }
        }

        if let Some(https) = https_proxy.or(http_proxy) {
            return ensure_proxy_scheme(https, "http://");
        }
        if let Some(socks) = socks_proxy {
            return ensure_proxy_scheme(socks, "socks5://");
        }
    }

    ensure_proxy_scheme(trimmed, "http://")
}

fn ensure_proxy_scheme(addr: &str, default_scheme: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else {
        format!("{}{}", default_scheme, addr)
    }
}

/// Builds a reqwest client with proper proxy and timeout configuration.
pub fn build_http_client(timeout: Duration) -> Result<Client, AppError> {
    let mut builder = ClientBuilder::new()
        .timeout(timeout);

    if let Some(proxy_url) = detect_system_proxy() {
        tracing::info!("Using system proxy for Google API requests: {}", proxy_url);
        if let Ok(proxy) = Proxy::all(&proxy_url) {
            builder = builder.proxy(proxy.no_proxy(reqwest::NoProxy::from_string("localhost,127.0.0.1,::1")));
        }
    }

    builder.build().map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_proxy_spec() {
        assert_eq!(normalize_proxy_spec("127.0.0.1:10808"), "http://127.0.0.1:10808");
        assert_eq!(normalize_proxy_spec("http://127.0.0.1:7890"), "http://127.0.0.1:7890");
        assert_eq!(normalize_proxy_spec("socks5://127.0.0.1:1080"), "socks5://127.0.0.1:1080");
        assert_eq!(
            normalize_proxy_spec("http=127.0.0.1:8080;https=127.0.0.1:8443"),
            "http://127.0.0.1:8443"
        );
        assert_eq!(
            normalize_proxy_spec("socks=127.0.0.1:10808"),
            "socks5://127.0.0.1:10808"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_detect_system_proxy_on_windows() {
        let proxy = detect_system_proxy();
        println!("Detected proxy: {:?}", proxy);
        assert!(proxy.is_some());
        assert_eq!(proxy.unwrap(), "http://127.0.0.1:10808");
    }

    #[tokio::test]
    async fn test_client_connects_to_google_with_proxy() {
        let client = build_http_client(Duration::from_secs(10)).expect("build client");
        let resp = client.get("https://www.googleapis.com/oauth2/v2/userinfo").send().await;
        println!("Response: {:?}", resp);
        assert!(resp.is_ok(), "Request through proxy should succeed to reach Google server");
        // Status should be 401 Unauthorized (because no token was sent), NOT timeout or connection error!
        assert_eq!(resp.unwrap().status(), 401);
    }
}
