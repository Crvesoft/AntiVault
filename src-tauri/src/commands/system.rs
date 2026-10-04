use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sysinfo::System;
use tauri::State;

use crate::error::AppError;
use crate::storage::account::Database;
use crate::utils::protobuf::build_state_sync_payload;
use crate::commands::account::SwitchResult;
use crate::commands::quota::refresh_access_token_via_oauth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityStatus {
    pub is_running: bool,
    pub executable_path: Option<String>,
    pub db_path: Option<String>,
    pub running_pids: Vec<u32>,
}

fn get_db_candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        for folder in &["Antigravity IDE", "Antigravity"] {
            paths.push(
                PathBuf::from(&appdata)
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

    #[cfg(target_os = "linux")]
    if let Some(home) = dirs::home_dir() {
        for folder in &["Antigravity IDE", "Antigravity"] {
            paths.push(
                home.join(".config")
                    .join(folder)
                    .join("User")
                    .join("globalStorage")
                    .join("state.vscdb"),
            );
        }
    }

    paths
}

pub fn locate_antigravity_db() -> Option<PathBuf> {
    for p in get_db_candidate_paths() {
        if p.exists() && p.is_file() {
            return Some(p);
        }
    }
    None
}

pub fn check_running_processes() -> (bool, Vec<u32>, Option<String>) {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut pids = Vec::new();
    let mut exe_path = None;

    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        if (name.contains("antigravity") || name == "agy" || name == "agy.exe") && !name.contains("antivault") {
            // Filter helper / renderer / crashpad if needed, but for is_running we track main PID
            let args = process.cmd()
                .iter()
                .map(|a| a.to_string_lossy().to_lowercase())
                .collect::<Vec<String>>()
                .join(" ");

            let is_helper = args.contains("--type=")
                || name.contains("crashpad")
                || name.contains("helper")
                || name.contains("plugin");

            let pid_u32 = pid.as_u32();
            pids.push(pid_u32);

            if !is_helper {
                if let Some(p) = process.exe() {
                    let path_str = p.to_string_lossy().to_string();
                    let lower = path_str.to_lowercase();
                    if lower.contains("antigravity") && !lower.contains("agy") {
                        exe_path = Some(path_str);
                    } else if exe_path.is_none() {
                        exe_path = Some(path_str);
                    }
                }
            }
        }
    }

    let is_running = !pids.is_empty();
    (is_running, pids, exe_path)
}

pub fn find_antigravity_exe() -> Option<String> {
    let (_, _, running_exe) = check_running_processes();
    if let Some(p) = running_exe {
        return Some(p);
    }

    #[cfg(target_os = "windows")]
    {
        let mut candidates = Vec::new();

        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            candidates.push(PathBuf::from(&local_appdata).join("Programs").join("Antigravity IDE").join("Antigravity IDE.exe"));
            candidates.push(PathBuf::from(&local_appdata).join("Programs").join("Antigravity").join("Antigravity.exe"));
        }

        for drive in &["C", "D", "E"] {
            for pf in &["Program Files", "Program Files (x86)"] {
                for name in &["Antigravity IDE", "Antigravity"] {
                    candidates.push(PathBuf::from(format!(r"{}:\{}\{}\{}.exe", drive, pf, name, name)));
                }
            }
        }

        for c in candidates {
            if c.exists() && c.is_file() {
                return Some(c.to_string_lossy().to_string());
            }
        }

        // Pure Rust PATH scan without running `where.exe` (which creates console windows)
        if let Some(path_var) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_var) {
                for name in &["Antigravity IDE.exe", "Antigravity.exe", "agy.exe"] {
                    let candidate = dir.join(name);
                    if candidate.is_file() {
                        return Some(candidate.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}

#[tauri::command]
pub async fn get_antigravity_status() -> Result<AntigravityStatus, AppError> {
    let (is_running, pids, detected_exe) = check_running_processes();
    let exe_path = detected_exe.or_else(find_antigravity_exe);
    let db_path = locate_antigravity_db().map(|p| p.to_string_lossy().to_string());

    Ok(AntigravityStatus {
        is_running,
        executable_path: exe_path,
        db_path,
        running_pids: pids,
    })
}

#[cfg(target_os = "windows")]
fn terminate_pid_native(pid: u32) -> bool {
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut std::ffi::c_void;
        fn TerminateProcess(hProcess: *mut std::ffi::c_void, uExitCode: u32) -> i32;
        fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    }
    const PROCESS_TERMINATE: u32 = 0x0001;

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return false;
        }
        let res = TerminateProcess(handle, 1);
        CloseHandle(handle);
        res != 0
    }
}

#[tauri::command]
pub async fn close_antigravity_process() -> Result<bool, AppError> {
    let (is_running, _, _) = check_running_processes();
    if !is_running {
        // Report honestly: nothing was running, so nothing was closed. The UI turns
        // this into a "未检测到可关闭的进程" hint instead of a false success.
        return Ok(false);
    }

    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut pids_to_kill: std::collections::HashSet<u32> = std::collections::HashSet::new();

    // 1. Collect all processes matching antigravity or agy (excluding antivault)
    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        if (name.contains("antigravity") || name == "agy" || name == "agy.exe") && !name.contains("antivault") {
            pids_to_kill.insert(pid.as_u32());
        }
    }

    if pids_to_kill.is_empty() {
        return Ok(false);
    }

    // 2. Also collect descendant child processes (tree termination)
    let mut added = true;
    while added {
        added = false;
        for (pid, process) in system.processes() {
            let pid_u32 = pid.as_u32();
            if !pids_to_kill.contains(&pid_u32) {
                if let Some(parent) = process.parent() {
                    if pids_to_kill.contains(&parent.as_u32()) {
                        let name = process.name().to_string_lossy().to_lowercase();
                        if !name.contains("antivault") {
                            pids_to_kill.insert(pid_u32);
                            added = true;
                        }
                    }
                }
            }
        }
    }

    // 3. Terminate processes natively - NO external taskkill / kill commands, NO flashing terminal windows
    for &pid in &pids_to_kill {
        #[cfg(target_os = "windows")]
        {
            terminate_pid_native(pid);
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Some(process) = system.process(sysinfo::Pid::from_u32(pid)) {
                let _ = process.kill();
            }
        }
    }

    // Poll until stopped (up to 3 seconds)
    for _ in 0..15 {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let (still_running, _, _) = check_running_processes();
        if !still_running {
            return Ok(true);
        }
    }

    let (still_running, _, _) = check_running_processes();
    Ok(!still_running)
}

#[tauri::command]
pub async fn launch_antigravity_app(exe_path: Option<String>) -> Result<(), AppError> {
    let target = exe_path.or_else(find_antigravity_exe);
    let exe = target.ok_or_else(|| AppError::AntigravityNotFound)?;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let exe_file = Path::new(&exe);
        let work_dir = exe_file.parent().unwrap_or_else(|| Path::new("."));

        Command::new(&exe)
            .current_dir(work_dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| AppError::Internal(format!("Failed to launch Antigravity: {}", e)))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new(&exe)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| AppError::Internal(format!("Failed to launch Antigravity: {}", e)))?;
    }

    Ok(())
}

fn backup_state_db(db_path: &Path) -> Result<PathBuf, AppError> {
    let now_str = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_name = format!("state.vscdb.backup.{}", now_str);
    let backup_path = db_path.with_file_name(backup_name);

    fs::copy(db_path, &backup_path)
        .map_err(|e| AppError::Internal(format!("Failed to backup state.vscdb: {}", e)))?;

    Ok(backup_path)
}

#[tauri::command]
pub async fn switch_account(
    account_id: String,
    auto_restart: Option<bool>,
    db: State<'_, Database>,
) -> Result<SwitchResult, AppError> {
    let restart = auto_restart.unwrap_or(false);

    // 1. Fetch account details from DB
    let (email, display_name, token_ref) = {
        let conn = db.conn()?;
        conn.query_row(
            "SELECT email, display_name, token_ref FROM accounts WHERE id = ?1",
            params![account_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?)),
        ).map_err(|_| AppError::AccountNotFound(account_id.clone()))?
    };

    // 2. Read refresh token from keyring
    let entry = keyring::Entry::new("antivault", &token_ref)
        .map_err(|e| AppError::SecureStorageError(e.to_string()))?;
    let refresh_token = entry.get_password()
        .map_err(|e| AppError::SecureStorageError(format!("Could not read refresh token: {}", e)))?;

    // 3. Refresh access token so we inject an immediately valid session
    let (access_token, expiry, id_token) = refresh_access_token_via_oauth(&refresh_token).await?;

    let (was_running, _, detected_exe) = check_running_processes();
    let exe_path = detected_exe.or_else(find_antigravity_exe);

    // If restart was requested and Antigravity is running, close it NOW before writing to state.vscdb
    // This avoids database lock contention and ensures Antigravity doesn't overwrite our newly injected token on exit!
    let mut restarted = false;
    if restart && was_running {
        let _ = close_antigravity_process().await;
        // Brief pause to allow OS file handles on state.vscdb to fully release
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    // 4. Update Antigravity 2.0 Credential in Windows Credential Manager (gemini:antigravity)
    #[cfg(target_os = "windows")]
    let v2_updated = {
        crate::utils::keyring_win::write_antigravity_20_credential(
            &access_token,
            &refresh_token,
            expiry,
            id_token.as_deref(),
        ).is_ok()
    };
    #[cfg(not(target_os = "windows"))]
    let v2_updated = false;

    // 5. If Antigravity IDE state database exists, backup & inject
    let ide_db_opt = locate_antigravity_db();
    if let Some(ref db_path) = ide_db_opt {
        // Backup DB
        let _backup = backup_state_db(db_path)?;

        // Build state sync payload
        let is_gcp_tos = false;
        let payload = build_state_sync_payload(
            &access_token,
            &refresh_token,
            expiry,
            id_token.as_deref(),
            is_gcp_tos,
            &email,
            display_name.as_deref(),
            None,
        );

        // Inject into SQLite (using WAL mode and 5-second busy timeout)
        {
            let vscdb = rusqlite::Connection::open(db_path)
                .map_err(|e| AppError::DatabaseError(format!("Failed to open state.vscdb: {}", e)))?;
            let _ = vscdb.busy_timeout(std::time::Duration::from_secs(5));

            vscdb.execute(
                "INSERT INTO ItemTable (key, value) VALUES ('antigravityUnifiedStateSync.oauthToken', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![payload],
            ).map_err(|e| AppError::SwitchFailed(format!("Failed to update oauthToken in ItemTable: {}", e)))?;
        }

        // Verify readback
        {
            let vscdb = rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(|e| AppError::DatabaseError(format!("Failed to reopen state.vscdb for verify: {}", e)))?;

            let readback: String = vscdb.query_row(
                "SELECT value FROM ItemTable WHERE key = 'antigravityUnifiedStateSync.oauthToken'",
                [],
                |row| row.get(0),
            ).map_err(|e| AppError::SwitchFailed(format!("Verify failed: {}", e)))?;

            if readback.trim() != payload.trim() {
                return Err(AppError::SwitchFailed("Verification failed: written token did not match readback".into()));
            }
        }
    } else if !v2_updated {
        return Err(AppError::SwitchFailed("Neither Antigravity 2.0 credential nor Antigravity IDE database was found".into()));
    }

    // 6. Set as current in AntiVault DB
    {
        let now = Utc::now().timestamp();
        let conn = db.conn()?;
        let _ = conn.execute("UPDATE accounts SET is_current = 0, updated_at = ?1", params![now]);
        let _ = conn.execute(
            "UPDATE accounts SET is_current = 1, last_used_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now, account_id],
        );
    }

    // 7. ONLY restart if restart was explicitly requested AND it was running
    if restart && was_running {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        if let Ok(()) = launch_antigravity_app(exe_path).await {
            restarted = true;
        }
    }


    Ok(SwitchResult {
        success: true,
        was_running,
        restarted,
        message: Some(format!("Successfully switched to {}", email)),
    })
}

#[tauri::command]
pub fn save_window_size(width: f64, height: f64, is_maximized: bool) {
    crate::utils::window_state::save_window_state(width, height, is_maximized);
}

#[tauri::command]
pub fn get_saved_window_size() -> Option<crate::utils::window_state::WindowState> {
    crate::utils::window_state::load_window_state()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processes() {
        let (is_running, pids, exe) = check_running_processes();
        println!("is_running: {}, pids: {:?}, exe: {:?}", is_running, pids, exe);
    }

    #[test]
    fn test_scan_db() {
        let db = locate_antigravity_db();
        println!("locate_antigravity_db: {:?}", db);
        if let Some(path) = db {
            let res = crate::commands::account::scan_antigravity_db(&path);
            println!("scan_antigravity_db result: {:?}", res);
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_create_no_window() {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let output = Command::new("cmd")
            .args(["/c", "echo hello"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        assert!(output.is_ok());
        let out = output.unwrap();
        let res = String::from_utf8_lossy(&out.stdout);
        assert!(res.contains("hello"));
    }

}





