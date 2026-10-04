use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{rng, RngCore};
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{Manager, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

use crate::commands::account::{add_account, AccountInfo, AddAccountRequest};
use crate::commands::quota::refresh_quota;
use crate::error::AppError;
use crate::storage::account::Database;

const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const USER_AGENT_VAL: &str = "antigravity/1.10.0 (Windows NT 10.0; Win64; x64)";
/// How long we keep the loopback listener alive waiting for the browser callback.
/// Real Google logins (typing credentials, 2FA prompts, "open email" flows) routinely
/// take more than three minutes, and the pairing is far more likely to be abandoned
/// than it is to conflict with another login, so a generous window is correct.
const LOGIN_TIMEOUT_SECS: u64 = 600;

const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>AntiVault Login Successful</title>
    <style>
        body { font-family: system-ui, -apple-system, sans-serif; background: #09090b; color: #f4f4f5; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
        .card { background: #18181b; padding: 2.5rem; border-radius: 1rem; border: 1px solid #27272a; text-align: center; max-width: 400px; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5); }
        h1 { color: #10b981; font-size: 1.5rem; margin-bottom: 0.5rem; }
        p { color: #a1a1aa; font-size: 0.95rem; line-height: 1.5; }
    </style>
</head>
<body>
    <div class="card">
        <h1>&#10003; Authorization Successful</h1>
        <p>Your Google account has been connected to AntiVault.<br>You can now safely close this tab.</p>
    </div>
</body>
</html>"#;

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    #[allow(dead_code)]
    expires_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
}

/// Why the pending login stopped waiting for the browser callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingOutcome {
    Waiting = 0,
    /// The user pasted the code/URL and `complete_google_login` finished the flow.
    CompletedManually = 1,
    /// The user dismissed the authorization modal.
    Cancelled = 2,
}

/// Marker returned to the frontend when the waiting `start_google_login` call is
/// unwound because the user finished the flow by pasting a code. The UI uses it to
/// suppress the "login failed" toast for the abandoned browser flow.
pub const MANUAL_COMPLETION_MARKER: &str = "__ANTIVAULT_MANUAL_COMPLETION__";

/// Marker returned when the waiting call is unwound because the user dismissed the
/// authorization modal. Not an error — the UI stays quiet about it.
pub const CANCELLED_MARKER: &str = "__ANTIVAULT_LOGIN_CANCELLED__";

/// PKCE material for the login that is currently waiting for the browser callback.
///
/// Kept in a process-wide slot so the user can finish the flow manually when the
/// browser cannot reach our loopback listener (proxy, firewall, `localhost`
/// resolving to a stack we are not listening on, ...).
struct PendingLogin {
    state: String,
    verifier: String,
    redirect_uri: String,
    /// Signalled when the flow is finished by another path (manual completion or
    /// cancel) so the waiting listener can stop instead of blocking until timeout.
    cancel: std::sync::Arc<tokio::sync::Notify>,
    /// Set alongside `cancel`; the listener re-checks it after every wake-up so a
    /// notification delivered between loop iterations cannot be lost.
    outcome: std::sync::Arc<std::sync::atomic::AtomicU8>,
}

static PENDING_LOGIN: std::sync::OnceLock<std::sync::Mutex<Option<PendingLogin>>> =
    std::sync::OnceLock::new();

fn pending_login() -> &'static std::sync::Mutex<Option<PendingLogin>> {
    PENDING_LOGIN.get_or_init(|| std::sync::Mutex::new(None))
}

fn clear_pending_login() {
    if let Ok(mut guard) = pending_login().lock() {
        *guard = None;
    }
}

/// Wakes the waiting listener and records why it should stop.
fn finish_pending(outcome: PendingOutcome) {
    if let Ok(guard) = pending_login().lock() {
        if let Some(p) = guard.as_ref() {
            p.outcome
                .store(outcome as u8, std::sync::atomic::Ordering::SeqCst);
            p.cancel.notify_waiters();
        }
    }
}

/// PKCE material for a recently started login flow, kept a little while after the
/// flow itself has ended (timeout / cancel / superseded by a new click / restart of
/// the waiting call). The single `PendingLogin` slot only knows the *current* flow,
/// but a code pasted manually may belong to an earlier one — the pasted URL carries
/// its own `state`, and that state is the key we look this record up with.
///
/// The record can also be persisted to disk so a paste still works after the app
/// process restarted between consent and the manual paste (e.g. a crash, or the OS
/// closing the app on logout). `created_at_unix_ms` is wall-clock rather than an
/// `Instant` so the serialized form is portable.
#[derive(Clone, Serialize, Deserialize)]
struct FlowRecord {
    state: String,
    verifier: String,
    redirect_uri: String,
    created_at_unix_ms: u64,
}

static RECENT_FLOWS: std::sync::OnceLock<std::sync::Mutex<Vec<FlowRecord>>> =
    std::sync::OnceLock::new();

/// Locked `Some(path)` once disk persistence has been initialised by a command.
/// `None` means "don't touch disk" — unit tests exercise the in-memory path only.
static RECENT_FLOWS_FILE: std::sync::OnceLock<std::sync::Mutex<Option<std::path::PathBuf>>> =
    std::sync::OnceLock::new();

/// How long a finished flow stays redeemable by state. Google auth codes are
/// single-use and expire quickly, so 30 minutes is a generous window.
const FLOW_RECORD_TTL: std::time::Duration = std::time::Duration::from_secs(30 * 60);
/// Cap on how many finished flows we remember.
const FLOW_RECORD_MAX: usize = 8;

fn recent_flows() -> &'static std::sync::Mutex<Vec<FlowRecord>> {
    RECENT_FLOWS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn prune_recent_flows(flows: &mut Vec<FlowRecord>) {
    let now = now_unix_ms();
    flows.retain(|f| now.saturating_sub(f.created_at_unix_ms) < FLOW_RECORD_TTL.as_millis() as u64);
    while flows.len() > FLOW_RECORD_MAX {
        flows.remove(0);
    }
}

/// Best-effort mirror of `RECENT_FLOWS` to disk, so pasting a callback URL still
/// resolves after the process restarted. Any I/O failure is silently ignored — the
/// in-memory store keeps working, it just won't survive a restart.
fn persist_recent_flows() {
    let file = match RECENT_FLOWS_FILE.get().and_then(|m| m.lock().ok()) {
        Some(guard) => match guard.as_ref() {
            Some(path) => path.clone(),
            None => return, // persistence not initialised (unit tests)
        },
        None => return,
    };
    if let Ok(flows) = recent_flows().lock() {
        if let Ok(json) = serde_json::to_string(&*flows) {
            let _ = std::fs::write(&file, json);
        }
    }
}

/// Points `RECENT_FLOWS_FILE` at the app data dir and loads whatever a previous
/// process left on disk into memory. Called once per process by the OAuth commands.
fn init_flows_persistence(app: &tauri::AppHandle) {
    let mut guard = match RECENT_FLOWS_FILE.get_or_init(|| std::sync::Mutex::new(None)).lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if guard.is_some() {
        return; // already initialised
    }
    let path = match app.path().app_data_dir() {
        Ok(dir) => dir.join("recent_flows.json"),
        Err(_) => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(disk_flows) = serde_json::from_str::<Vec<FlowRecord>>(&text) {
            if let Ok(mut flows) = recent_flows().lock() {
                for f in disk_flows {
                    if !flows.iter().any(|x| x.state == f.state) {
                        flows.push(f);
                    }
                }
                prune_recent_flows(&mut flows);
            }
        }
    }
    *guard = Some(path);
}

fn remember_flow(state: String, verifier: String, redirect_uri: String) {
    if let Ok(mut flows) = recent_flows().lock() {
        // A re-click on the login button generates a fresh state, so replacing an
        // existing entry with the same state is safe and avoids unbounded growth.
        flows.retain(|f| f.state != state);
        flows.push(FlowRecord {
            state,
            verifier,
            redirect_uri,
            created_at_unix_ms: now_unix_ms(),
        });
        prune_recent_flows(&mut flows);
    }
    persist_recent_flows();
}

fn find_flow(state: &str) -> Option<FlowRecord> {
    let flows = recent_flows().lock().ok()?;
    flows.iter().find(|f| f.state == state).cloned()
}

fn latest_flow() -> Option<FlowRecord> {
    let flows = recent_flows().lock().ok()?;
    flows.last().cloned()
}

fn remove_flow(state: &str) {
    let mut removed = false;
    if let Ok(mut flows) = recent_flows().lock() {
        let before = flows.len();
        flows.retain(|f| f.state != state);
        removed = flows.len() != before;
    }
    if removed {
        persist_recent_flows();
    }
}

/// Pulls the value of a single query parameter out of a callback URL.
fn query_param(input: &str, key: &str) -> Option<String> {
    let query = input.trim().split_once('?').map(|(_, q)| q)?;
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(urlencoding_decode(v));
            }
        }
    }
    None
}

/// Pulls `code` out of either a bare authorization code or the full redirect URL
/// that the browser ends up on when the loopback callback could not be reached.
fn extract_code(input: &str) -> Option<String> {
    let trimmed = input.trim().trim_matches('"').trim_matches('\'').trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(code) = query_param(trimmed, "code") {
        return Some(code);
    }

    // Strip "code=" prefix if user copied just "code=4/..." or "code=4%2F..."
    let s = if let Some(stripped) = trimmed.strip_prefix("code=") {
        stripped
    } else {
        trimmed
    };

    let s = s.split('#').next().unwrap_or(s);
    let decoded = urlencoding_decode(s);
    if decoded.starts_with("4/") || decoded.starts_with("1/") {
        Some(decoded)
    } else if s.starts_with("4/") || s.starts_with("1/") {
        Some(s.to_string())
    } else {
        None
    }
}

fn generate_pkce() -> (String, String) {
    let mut bytes = [0u8; 32];
    rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    (verifier, challenge)
}

fn failure_html(message: &str) -> String {
    let escaped = message
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>AntiVault Authorization Failed</title>
    <style>
        body {{ font-family: system-ui, -apple-system, sans-serif; background: #09090b; color: #f4f4f5; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }}
        .card {{ background: #18181b; padding: 2.5rem; border-radius: 1rem; border: 1px solid #27272a; text-align: center; max-width: 420px; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5); }}
        h1 {{ color: #f43f5e; font-size: 1.4rem; margin-bottom: 0.5rem; }}
        p {{ color: #a1a1aa; font-size: 0.95rem; line-height: 1.5; word-break: break-word; }}
    </style>
</head>
<body>
    <div class="card">
        <h1>&#10007; Authorization Failed</h1>
        <p>{}</p>
        <p>You can close this tab and retry inside AntiVault.</p>
    </div>
</body>
</html>"#,
        escaped
    )
}

async fn write_http_response(stream: &mut TcpStream, status: u16, body: &str) {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        _ => "Not Found",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n{}",
        status,
        reason,
        body.as_bytes().len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
    let _ = stream.shutdown().await;
}

/// Accepts a connection from either the IPv4 or the IPv6 loopback listener.
///
/// Windows resolves `localhost` to `::1` first, so a listener bound only to
/// `127.0.0.1` never sees the browser callback.
async fn accept_loopback(
    v4: &TcpListener,
    v6: Option<&TcpListener>,
) -> Result<(TcpStream, std::net::SocketAddr), AppError> {
    match v6 {
        Some(v6) => tokio::select! {
            r = v4.accept() => r.map_err(|e| AppError::Internal(format!("Loopback accept failed: {}", e))),
            r = v6.accept() => r.map_err(|e| AppError::Internal(format!("Loopback accept failed: {}", e))),
        },
        None => v4
            .accept()
            .await
            .map_err(|e| AppError::Internal(format!("Loopback accept failed: {}", e))),
    }
}

#[tauri::command]
pub async fn start_google_login(
    app: tauri::AppHandle,
    db: State<'_, Database>,
) -> Result<AccountInfo, AppError> {
    use tauri_plugin_opener::OpenerExt;

    // Load any flows a previous process persisted, so a paste still works after a
    // restart between consent and the manual paste.
    init_flows_persistence(&app);

    // 1. Bind both loopback stacks on the same port so `http://localhost:PORT`
    //    works regardless of whether the browser picks IPv4 or IPv6.
    //    Windows resolves `localhost` to `::1` first, so binding only 127.0.0.1
    //    makes the browser callback fail and the flow hang until it times out.
    let listener_v4 = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| AppError::Internal(format!("Failed to bind local port: {}", e)))?;
    let port = listener_v4
        .local_addr()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .port();
    let listener_v6 = TcpListener::bind(("::1", port)).await.ok();

    // When both stacks listen, `localhost` is safe. When only one is bound, use the
    // explicit address so the browser cannot pick the stack we are not listening on.
    let redirect_uri = match listener_v6 {
        Some(_) => format!("http://localhost:{}/oauth-callback", port),
        None => format!("http://127.0.0.1:{}/oauth-callback", port),
    };
    let (verifier, challenge) = generate_pkce();
    let state = Uuid::new_v4().to_string();

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
        client_id={}&\
        response_type=code&\
        scope=openid%20email%20profile%20https://www.googleapis.com/auth/cloud-platform%20https://www.googleapis.com/auth/cclog%20https://www.googleapis.com/auth/experimentsandconfigs&\
        redirect_uri={}&\
        code_challenge={}&\
        code_challenge_method=S256&\
        state={}&\
        access_type=offline&\
        prompt=consent&\
        include_granted_scopes=true",
        crate::utils::get_oauth_client_id(),
        urlencoding_encode(&redirect_uri),
        challenge,
        state
    );

    // 2. Remember the PKCE verifier so a manual code/URL submission can finish the
    //    flow even if the browser never reaches our loopback listener.
    let cancel = std::sync::Arc::new(tokio::sync::Notify::new());
    let outcome_flag = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(
        PendingOutcome::Waiting as u8,
    ));
    {
        let mut guard = pending_login()
            .lock()
            .map_err(|_| AppError::Internal("OAuth state lock corrupted".into()))?;
        *guard = Some(PendingLogin {
            state: state.clone(),
            verifier: verifier.clone(),
            redirect_uri: redirect_uri.clone(),
            cancel: cancel.clone(),
            outcome: outcome_flag.clone(),
        });
    }
    // Also remember this flow in the recent-flow history so a code pasted after this
    // flow has ended (timeout / cancel / superseded) can still be redeemed by state.
    remember_flow(state.clone(), verifier.clone(), redirect_uri.clone());

    // 3. Open browser
    app.opener()
        .open_url(&auth_url, None::<&str>)
        .map_err(|e| AppError::Internal(format!("Failed to open browser: {}", e)))?;

    // 4. Await callback with a timeout
    let outcome = tokio::time::timeout(std::time::Duration::from_secs(LOGIN_TIMEOUT_SECS), async {
        loop {
            // Re-check the shared outcome on every iteration: a wake-up that lands
            // between two `select!` polls must not be lost.
            match outcome_flag.load(std::sync::atomic::Ordering::SeqCst) {
                x if x == PendingOutcome::CompletedManually as u8 => {
                    return Err(AppError::AuthenticationFailed(
                        MANUAL_COMPLETION_MARKER.to_string(),
                    ));
                }
                x if x == PendingOutcome::Cancelled as u8 => {
                    return Err(AppError::AuthenticationFailed(
                        CANCELLED_MARKER.to_string(),
                    ));
                }
                _ => {}
            }

            let accepted = tokio::select! {
                r = accept_loopback(&listener_v4, listener_v6.as_ref()) => r?,
                _ = cancel.notified() => continue,
            };
            let (mut stream, _peer) = accepted;

            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            if n == 0 {
                continue;
            }
            let request = String::from_utf8_lossy(&buf[..n]).to_string();

            let first_line = match request.lines().next() {
                Some(line) => line.to_string(),
                None => continue,
            };
            let target = first_line.split_whitespace().nth(1).unwrap_or("");
            let (path, query) = match target.split_once('?') {
                Some((path, query)) => (path, query),
                None => (target, ""),
            };

            if path != "/oauth-callback" {
                write_http_response(&mut stream, 404, "<h1>404 Not Found</h1>").await;
                continue;
            }

            let mut code_param: Option<String> = None;
            let mut state_param: Option<String> = None;
            let mut error_param: Option<String> = None;
            let mut error_desc: Option<String> = None;

            for pair in query.split('&') {
                if pair.is_empty() {
                    continue;
                }
                let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
                let value = urlencoding_decode(value);
                match key {
                    "code" => code_param = Some(value),
                    "state" => state_param = Some(value),
                    "error" => error_param = Some(value),
                    "error_description" => error_desc = Some(value),
                    _ => {}
                }
            }

            // Google reported an error (e.g. access_denied) instead of a code.
            if let Some(err) = error_param {
                let detail = error_desc.unwrap_or_else(|| "no description provided".to_string());
                write_http_response(
                    &mut stream,
                    200,
                    &failure_html(&format!("Google returned `{}`: {}", err, detail)),
                )
                .await;
                return Err(AppError::AuthenticationFailed(format!(
                    "Google OAuth error: {} ({})",
                    err, detail
                )));
            }

            // CSRF protection: the state echoed back must match the one we sent.
            if state_param.as_deref() != Some(state.as_str()) {
                write_http_response(
                    &mut stream,
                    400,
                    &failure_html("state parameter mismatch - authorization rejected."),
                )
                .await;
                continue;
            }

            if let Some(code) = code_param {
                write_http_response(&mut stream, 200, SUCCESS_HTML).await;
                return Ok::<String, AppError>(code);
            }

            write_http_response(
                &mut stream,
                400,
                &failure_html("No authorization code was returned by Google."),
            )
            .await;
        }
        #[allow(unreachable_code)]
        Ok::<String, AppError>(String::new())
    })
    .await
    .map_err(|_| {
        AppError::AuthenticationFailed(format!(
            "Login timed out after {} seconds",
            LOGIN_TIMEOUT_SECS
        ))
    })?;

    let code = match outcome {
        Ok(code) => {
            clear_pending_login();
            // The code is single-use; drop the history entry too so the same URL
            // cannot be re-pasted and replayed.
            remove_flow(&state);
            code
        }
        Err(e) => {
            clear_pending_login();
            return Err(e);
        }
    };

    // 5. Exchange the code and persist the account.
    finish_login(code, &verifier, &redirect_uri, db).await
}

/// Exchanges an authorization code for tokens, stores the account and returns it.
///
/// Shared by the loopback callback path and the manual completion path.
async fn finish_login(
    code: String,
    verifier: &str,
    redirect_uri: &str,
    db: State<'_, Database>,
) -> Result<AccountInfo, AppError> {
    let client = crate::utils::http::build_http_client(std::time::Duration::from_secs(30))?;
    let client_id = crate::utils::get_oauth_client_id();
    let client_secret = crate::utils::get_oauth_client_secret();

    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("code", code.as_str()),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
        ("code_verifier", verifier),
    ];

    let resp = client
        .post(TOKEN_URL)
        .header(USER_AGENT, USER_AGENT_VAL)
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::AuthenticationFailed(format!("Token exchange failed: {}", e)))?;

    if !resp.status().is_success() {
        let err_body = resp.text().await.unwrap_or_default();
        return Err(AppError::AuthenticationFailed(format!("Token error: {}", err_body)));
    }

    let token_data: GoogleTokenResponse = resp.json().await
        .map_err(|e| AppError::AuthenticationFailed(format!("Failed to parse token response: {}", e)))?;

    let refresh_token = token_data.refresh_token.ok_or_else(|| {
        AppError::AuthenticationFailed("No refresh token returned by Google OAuth. Try removing AntiVault app access in your Google Account and log in again.".into())
    })?;

    // Fetch user profile info
    let user_info_resp = client
        .get(USERINFO_URL)
        .header(AUTHORIZATION, format!("Bearer {}", token_data.access_token))
        .header(USER_AGENT, USER_AGENT_VAL)
        .send()
        .await
        .map_err(|e| AppError::AuthenticationFailed(format!("Userinfo request failed: {}", e)))?;

    if !user_info_resp.status().is_success() {
        let err_body = user_info_resp.text().await.unwrap_or_default();
        return Err(AppError::AuthenticationFailed(format!(
            "Userinfo error: {}",
            err_body
        )));
    }

    let user_info: GoogleUserInfo = user_info_resp.json().await
        .map_err(|e| AppError::AuthenticationFailed(format!("Failed to parse userinfo: {}", e)))?;

    // Save or update account in database (re-login with the same email updates it)
    let add_req = AddAccountRequest {
        email: user_info.email.clone(),
        display_name: user_info.name.clone(),
        refresh_token,
        access_token: Some(token_data.access_token),
        subscription_type: None,
    };

    let account = add_account(add_req, db.clone()).await?;

    // Initial quota fetch (best effort — never fails the login)
    let _ = refresh_quota(account.id.clone(), db.clone()).await;

    // Return the fresh account with quota status
    crate::commands::account::get_account(account.id, db).await
}

/// Finishes a login that was started by [`start_google_login`] using a code (or the
/// full redirect URL) pasted by the user.
///
/// This is the fallback for environments where the browser cannot reach the local
/// callback listener at all.
///
/// The pasted text may belong to *any* flow this process started recently, not just
/// the one that is currently pending (the user may have timed out / cancelled / re-clicked
/// before pasting). The URL carries its own `state`, which is the key we match against
/// both the current pending login and the recent-flow history. The CSRF guard stays
/// intact: only states that WE generated are ever accepted.
#[tauri::command]
pub async fn complete_google_login(
    app: tauri::AppHandle,
    code_or_url: String,
    db: State<'_, Database>,
) -> Result<AccountInfo, AppError> {
    // Ensure the recent-flow history is loaded from disk: the pasted URL may belong
    // to a flow started before the app was restarted.
    init_flows_persistence(&app);

    let code = extract_code(&code_or_url).ok_or_else(|| {
        AppError::AuthenticationFailed(
            "Could not find an authorization code in the pasted text. Paste the full callback URL or the raw code."
                .into(),
        )
    })?;

    // A raw code has no state; only a full URL tells us which flow it belongs to.
    let pasted_state = query_param(&code_or_url, "state");

    let (verifier, redirect_uri) = {
        // 1. If the current pending flow matches (raw code with no state also falls
        //    back here, as before), use it.
        let guard = pending_login()
            .lock()
            .map_err(|_| AppError::Internal("OAuth state lock corrupted".into()))?;
        let (pending_state, pending_verifier, pending_redirect) = match &*guard {
            Some(p) => (
                Some(p.state.clone()),
                Some(p.verifier.clone()),
                Some(p.redirect_uri.clone()),
            ),
            None => (None, None, None),
        };
        drop(guard);
        match pending_verifier {
            Some(v) if pasted_state.as_deref().is_none_or(|s| s == pending_state.as_deref().unwrap_or_default()) => {
                (v, pending_redirect.expect("redirect_uri set with verifier"))
            }
            // 2. Otherwise the pasted URL must carry a state we generated for an
            //    earlier flow (timeout/cancel/superseded). Look it up in history.
            _ => {
                let flow = pasted_state
                    .as_deref()
                    .and_then(find_flow)
                    .or_else(|| {
                        if pasted_state.is_none() {
                            latest_flow()
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| {
                        if pasted_state.is_none() {
                            AppError::AuthenticationFailed(
                                "No authorization is in progress. Click \"打开浏览器进行授权登录\" first, then paste the code."
                                    .into(),
                            )
                        } else {
                            AppError::AuthenticationFailed(
                                "state parameter mismatch - authorization rejected. \
                                 The pasted link belongs to a flow this app no longer remembers \
                                 (it may have expired, been cancelled, or the app restarted). \
                                 Click \"打开浏览器进行授权登录\" to start a fresh login and paste the new link."
                                    .into(),
                            )
                        }
                    })?;
                (flow.verifier, flow.redirect_uri)
            }
        }
    };

    let account = finish_login(code, &verifier, &redirect_uri, db).await?;

    // The flow is finished; wake the waiting listener, drop the pending material and
    // forget this (single-use) flow so the code cannot be replayed.
    if let Some(state) = pasted_state {
        remove_flow(&state);
    }
    finish_pending(PendingOutcome::CompletedManually);
    clear_pending_login();

    Ok(account)
}

/// Dismisses the pending login (used when the user closes the authorization modal).
#[tauri::command]
pub async fn cancel_google_login() -> Result<(), AppError> {
    finish_pending(PendingOutcome::Cancelled);
    clear_pending_login();
    Ok(())
}

fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

fn urlencoding_decode(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 3 <= bytes.len() {
            if let Ok(val) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                result.push(val);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            result.push(b' ');
            i += 1;
            continue;
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&result).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencoding_roundtrip() {
        let raw = "http://localhost:1234/oauth-callback";
        assert_eq!(urlencoding_decode(&urlencoding_encode(raw)), raw);
    }

    #[test]
    fn test_urlencoding_decode_trailing_escape() {
        // A %XX sequence at the very end of the string must still be decoded.
        assert_eq!(urlencoding_decode("a%2F"), "a/");
    }

    #[test]
    fn test_pkce_pair() {
        let (verifier, challenge) = generate_pkce();
        assert_eq!(verifier.len(), 43);
        assert_eq!(challenge.len(), 43);
        assert_ne!(verifier, challenge);
    }

    #[test]
    fn test_extract_code_from_full_url() {
        let url = "http://localhost:51234/oauth-callback?state=abc-123&code=4%2F0Abc-_xyz&scope=email";
        assert_eq!(extract_code(url).as_deref(), Some("4/0Abc-_xyz"));
    }

    #[test]
    fn test_extract_code_from_raw_code() {
        assert_eq!(extract_code("  4/0Aabcdef  ").as_deref(), Some("4/0Aabcdef"));
        assert_eq!(extract_code("1/abcdef").as_deref(), Some("1/abcdef"));
        assert_eq!(extract_code("4%2F0Aabcdef").as_deref(), Some("4/0Aabcdef"));
        assert_eq!(extract_code("code=4%2F0Aabcdef").as_deref(), Some("4/0Aabcdef"));
        assert_eq!(extract_code("\"4/0Aabcdef\"").as_deref(), Some("4/0Aabcdef"));
    }

    #[test]
    fn test_extract_code_rejects_junk() {
        assert_eq!(extract_code(""), None);
        assert_eq!(extract_code("   "), None);
        assert_eq!(extract_code("not-a-code"), None);
    }

    #[test]
    fn test_query_param_finds_state() {
        let url = "http://localhost:10587/oauth-callback?state=e5584643-3b47-463a-ace3-86681e118981&code=4%2F0Axyz&scope=email";
        assert_eq!(
            query_param(url, "state").as_deref(),
            Some("e5584643-3b47-463a-ace3-86681e118981")
        );
        assert_eq!(query_param(url, "code").as_deref(), Some("4/0Axyz"));
        assert_eq!(query_param(url, "missing"), None);
        assert_eq!(query_param("not-a-url", "state"), None);
    }

    #[test]
    fn test_flow_history_remembers_finds_and_removes() {
        remove_flow("hist-state-a");
        remove_flow("hist-state-b");

        remember_flow(
            "hist-state-a".into(),
            "verifier-a".into(),
            "http://localhost:10587/oauth-callback".into(),
        );
        remember_flow(
            "hist-state-b".into(),
            "verifier-b".into(),
            "http://localhost:10587/oauth-callback".into(),
        );

        let a = find_flow("hist-state-a").expect("flow a remembered");
        assert_eq!(a.verifier, "verifier-a");
        assert_eq!(a.redirect_uri, "http://localhost:10587/oauth-callback");

        // Re-remembering the same state replaces, not duplicates.
        remember_flow(
            "hist-state-a".into(),
            "verifier-a2".into(),
            "http://localhost:10587/oauth-callback".into(),
        );
        assert_eq!(find_flow("hist-state-a").expect("flow a replaced").verifier, "verifier-a2");

        remove_flow("hist-state-a");
        assert!(find_flow("hist-state-a").is_none(), "removed flow is gone");
        assert!(find_flow("hist-state-b").is_some(), "sibling flow untouched");

        remove_flow("hist-state-b");
        assert!(find_flow("hist-state-b").is_none());
    }

    #[test]
    fn test_flow_record_serde_roundtrip() {
        let f = FlowRecord {
            state: "state-xyz".into(),
            verifier: "verifier-xyz".into(),
            redirect_uri: "http://localhost:51234/oauth-callback".into(),
            created_at_unix_ms: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&f).expect("serialize");
        let back: FlowRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.state, "state-xyz");
        assert_eq!(back.verifier, "verifier-xyz");
        assert_eq!(back.redirect_uri, "http://localhost:51234/oauth-callback");
        assert_eq!(back.created_at_unix_ms, 1_700_000_000_000);
    }

    /// The whole point of the dual-stack bind: a browser resolving `localhost` to
    /// `::1` must still reach us. Both stacks are exercised through `accept_loopback`.
    #[tokio::test]
    async fn test_accept_loopback_serves_both_stacks() {
        let v4 = TcpListener::bind("127.0.0.1:0").await.expect("bind v4");
        let port = v4.local_addr().expect("v4 addr").port();
        let v6 = TcpListener::bind(("::1", port)).await.ok();

        // IPv4 client
        let h4 = tokio::spawn(async move {
            let _ = tokio::net::TcpStream::connect(("127.0.0.1", port)).await;
        });
        let (mut s4, _) = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            accept_loopback(&v4, v6.as_ref()),
        )
        .await
        .expect("v4 accept timed out")
        .expect("v4 accept failed");
        let _ = s4.shutdown().await;
        h4.await.expect("v4 client task");

        // IPv6 client — only meaningful when the second stack bound successfully.
        if let Some(v6) = v6 {
            let h6 = tokio::spawn(async move {
                let _ = tokio::net::TcpStream::connect(("::1", port)).await;
            });
            let (mut s6, _) = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                accept_loopback(&v4, Some(&v6)),
            )
            .await
            .expect("v6 accept timed out")
            .expect("v6 accept failed");
            let _ = s6.shutdown().await;
            h6.await.expect("v6 client task");
        }
    }
}
