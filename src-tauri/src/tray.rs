use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use crate::storage::account::Database;

pub const TRAY_ID: &str = "main-tray";
static IS_QUITTING: AtomicBool = AtomicBool::new(false);
static CLOSE_TO_TRAY: AtomicBool = AtomicBool::new(true);

pub fn is_quitting() -> bool {
    IS_QUITTING.load(Ordering::SeqCst)
}

pub fn set_quitting(quitting: bool) {
    IS_QUITTING.store(quitting, Ordering::SeqCst);
}

pub fn get_close_to_tray() -> bool {
    CLOSE_TO_TRAY.load(Ordering::SeqCst)
}

pub fn set_close_to_tray(enabled: bool) {
    CLOSE_TO_TRAY.store(enabled, Ordering::SeqCst);
}

#[cfg(target_os = "windows")]
pub fn show_tray_message(title: &str, msg: &str, is_error: bool) {
    extern "system" {
        fn MessageBoxW(hwnd: isize, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
    }
    let wide_title: Vec<u16> = format!("{}\0", title).encode_utf16().collect();
    let wide_msg: Vec<u16> = format!("{}\0", msg).encode_utf16().collect();
    let icon_flag = if is_error { 0x10 } else { 0x40 }; // MB_ICONERROR or MB_ICONINFORMATION
    unsafe {
        MessageBoxW(0, wide_msg.as_ptr(), wide_title.as_ptr(), icon_flag);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn show_tray_message(_title: &str, msg: &str, is_error: bool) {
    if is_error {
        tracing::error!("{}", msg);
    } else {
        tracing::info!("{}", msg);
    }
}

fn format_account_label(email: &str, display_name: Option<&str>, sub_type: Option<&str>) -> String {
    let type_str = sub_type.unwrap_or("FREE").to_uppercase();
    if let Some(name) = display_name {
        if !name.is_empty() && name != email {
            format!("{} ({}) [{}]", name, email, type_str)
        } else {
            format!("{} [{}]", email, type_str)
        }
    } else {
        format!("{} [{}]", email, type_str)
    }
}

pub fn create_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let menu = Menu::new(app)?;

    let db = app.try_state::<Database>();
    let (accounts, current_email) = if let Some(ref db) = db {
        let conn = db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, email, display_name, subscription_type, is_current FROM accounts ORDER BY sort_order ASC, created_at ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i32>(4)? != 0,
            ))
        })?;
        let mut list = Vec::new();
        let mut curr = None;
        for r in rows.flatten() {
            if r.4 {
                curr = Some(r.1.clone());
            }
            list.push(r);
        }
        (list, curr)
    } else {
        (Vec::new(), None)
    };

    // Header info
    let title_text = match current_email {
        Some(ref email) => format!("AntiVault (当前: {})", email),
        None => "AntiVault (未连接账号)".to_string(),
    };
    let title_item = MenuItem::with_id(app, "header_title", title_text, false, None::<&str>)?;
    menu.append(&title_item)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Accounts section
    if accounts.is_empty() {
        let no_acc = MenuItem::with_id(app, "no_accounts", "（暂无账号，点击主界面添加）", false, None::<&str>)?;
        menu.append(&no_acc)?;
    } else if accounts.len() <= 8 {
        for (id, email, display_name, sub_type, is_current) in &accounts {
            let label = format_account_label(email, display_name.as_deref(), sub_type.as_deref());
            let menu_id = format!("switch_account:{}", id);
            let item = CheckMenuItem::with_id(app, &menu_id, label, true, *is_current, None::<&str>)?;
            menu.append(&item)?;
        }
    } else {
        let sub = Submenu::with_id(app, "switch_submenu", "切换账号", true)?;
        for (id, email, display_name, sub_type, is_current) in &accounts {
            let label = format_account_label(email, display_name.as_deref(), sub_type.as_deref());
            let menu_id = format!("switch_account:{}", id);
            let item = CheckMenuItem::with_id(app, &menu_id, label, true, *is_current, None::<&str>)?;
            sub.append(&item)?;
        }
        menu.append(&sub)?;
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    let show_item = MenuItem::with_id(app, "show_window", "打开 AntiVault 主界面", true, None::<&str>)?;
    menu.append(&show_item)?;

    let restart_item = MenuItem::with_id(app, "restart_antigravity", "启动 / 重启 Antigravity 客户端", true, None::<&str>)?;
    menu.append(&restart_item)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    let quit_item = MenuItem::with_id(app, "quit", "退出 AntiVault", true, None::<&str>)?;
    menu.append(&quit_item)?;

    Ok(menu)
}

pub fn update_tray_menu(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Ok(menu) = create_tray_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }

        // Update tray tooltip with current active account
        if let Some(db) = app.try_state::<Database>() {
            if let Ok(conn) = db.conn() {
                let current_email: Result<String, _> = conn.query_row(
                    "SELECT email FROM accounts WHERE is_current = 1 LIMIT 1",
                    [],
                    |row| row.get(0),
                );
                let tooltip = match current_email {
                    Ok(email) => format!("AntiVault - 当前账号: {}", email),
                    Err(_) => "AntiVault - Antigravity 多账号管理".to_string(),
                };
                let _ = tray.set_tooltip(Some(tooltip));
            }
        }
    }
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = create_tray_menu(app)?;

    let icon = match app.default_window_icon().cloned() {
        Some(ic) => ic,
        None => tauri::image::Image::from_app_icon_resource(32)?,
    };

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("AntiVault - Antigravity 多账号管理")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let id_str = event.id().as_ref();
            match id_str {
                "show_window" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
                "restart_antigravity" => {
                    tauri::async_runtime::spawn(async move {
                        let (is_running, _, detected_exe) = crate::commands::system::check_running_processes();
                        let exe = detected_exe.or_else(crate::commands::system::find_antigravity_exe);
                        if is_running {
                            let _ = crate::commands::system::close_antigravity_process().await;
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        }
                        let _ = crate::commands::system::launch_antigravity_app(exe).await;
                    });
                }
                "quit" => {
                    set_quitting(true);
                    app.exit(0);
                }
                id if id.starts_with("switch_account:") => {
                    let account_id = id.trim_start_matches("switch_account:").to_string();
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Some(db) = app_clone.try_state::<Database>() {
                            match crate::commands::system::switch_account(account_id.clone(), Some(true), db, app_clone.clone()).await {
                                Ok(res) => {
                                    tracing::info!("Tray switched account {}: {:?}", account_id, res);
                                    update_tray_menu(&app_clone);
                                    let _ = app_clone.emit("account-switched", account_id);
                                }
                                Err(e) => {
                                    tracing::error!("Tray failed to switch account {}: {}", account_id, e);
                                    show_tray_message("AntiVault - 账号切换失败", &format!("无法切换账号: {}", e), true);
                                }
                            }
                        }
                    });
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
                | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        })
        .build(app)?;

    // Initial tooltip update with current account
    update_tray_menu(app);

    // Initialize close_to_tray setting
    let settings = crate::utils::settings::load_settings();
    set_close_to_tray(settings.close_to_tray);

    Ok(())
}
