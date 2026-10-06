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

#[derive(Debug, Clone)]
pub struct TrayAccountInfo {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub is_current: bool,
    pub g_5h: Option<f64>,
    pub g_weekly: Option<f64>,
    pub c_5h: Option<f64>,
    pub c_weekly: Option<f64>,
}

fn format_account_label(acc: &TrayAccountInfo) -> String {
    let name = acc
        .display_name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| acc.email.split('@').next().unwrap_or(&acc.email));

    let h5 = acc.g_5h.or(acc.c_5h);
    let weekly = acc.g_weekly.or(acc.c_weekly);

    match (h5, weekly) {
        (Some(h), Some(w)) => format!("{}  {:.0}% ({:.0}%)", name, h, w),
        (Some(h), None) => format!("{}  {:.0}%", name, h),
        _ => format!("{}  --% (--%)", name),
    }
}

pub fn create_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let menu = Menu::new(app)?;

    let db = app.try_state::<Database>();
    let (accounts, current_label) = if let Some(ref db) = db {
        let conn = db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT 
                a.id, 
                a.email, 
                a.display_name, 
                a.is_current,
                MAX(CASE WHEN q.model_name = 'Gemini (5h)' THEN q.remaining_percent END),
                MAX(CASE WHEN q.model_name = 'Gemini (Weekly)' THEN q.remaining_percent END),
                MAX(CASE WHEN q.model_name = 'Claude (5h)' THEN q.remaining_percent END),
                MAX(CASE WHEN q.model_name = 'Claude (Weekly)' THEN q.remaining_percent END)
            FROM accounts a
            LEFT JOIN quotas q ON a.id = q.account_id
            GROUP BY a.id
            ORDER BY a.sort_order ASC, a.created_at ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TrayAccountInfo {
                id: row.get(0)?,
                email: row.get(1)?,
                display_name: row.get(2)?,
                is_current: row.get::<_, i32>(3)? != 0,
                g_5h: row.get(4)?,
                g_weekly: row.get(5)?,
                c_5h: row.get(6)?,
                c_weekly: row.get(7)?,
            })
        })?;
        let mut list = Vec::new();
        let mut curr_lbl = None;
        for r in rows.flatten() {
            if r.is_current {
                curr_lbl = Some(format_account_label(&r));
            }
            list.push(r);
        }
        (list, curr_lbl)
    } else {
        (Vec::new(), None)
    };

    // Header info
    let title_text = match current_label {
        Some(ref lbl) => format!("AntiVault (当前: {})", lbl),
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
        for acc in &accounts {
            let label = format_account_label(acc);
            let menu_id = format!("switch_account:{}", acc.id);
            let item = CheckMenuItem::with_id(app, &menu_id, label, true, acc.is_current, None::<&str>)?;
            menu.append(&item)?;
        }
    } else {
        let sub = Submenu::with_id(app, "switch_submenu", "切换账号", true)?;
        for acc in &accounts {
            let label = format_account_label(acc);
            let menu_id = format!("switch_account:{}", acc.id);
            let item = CheckMenuItem::with_id(app, &menu_id, label, true, acc.is_current, None::<&str>)?;
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
                let stmt_res = conn.prepare(
                    "SELECT 
                        a.id, 
                        a.email, 
                        a.display_name, 
                        a.is_current,
                        MAX(CASE WHEN q.model_name = 'Gemini (5h)' THEN q.remaining_percent END),
                        MAX(CASE WHEN q.model_name = 'Gemini (Weekly)' THEN q.remaining_percent END),
                        MAX(CASE WHEN q.model_name = 'Claude (5h)' THEN q.remaining_percent END),
                        MAX(CASE WHEN q.model_name = 'Claude (Weekly)' THEN q.remaining_percent END)
                    FROM accounts a
                    LEFT JOIN quotas q ON a.id = q.account_id
                    WHERE a.is_current = 1
                    GROUP BY a.id
                    LIMIT 1"
                );
                if let Ok(mut stmt) = stmt_res {
                    let current_acc = stmt.query_row([], |row| {
                        Ok(TrayAccountInfo {
                            id: row.get(0)?,
                            email: row.get(1)?,
                            display_name: row.get(2)?,
                            is_current: true,
                            g_5h: row.get(4)?,
                            g_weekly: row.get(5)?,
                            c_5h: row.get(6)?,
                            c_weekly: row.get(7)?,
                        })
                    });
                    let tooltip = match current_acc {
                        Ok(acc) => format!("AntiVault - 当前: {}", format_account_label(&acc)),
                        Err(_) => "AntiVault - Antigravity 多账号管理".to_string(),
                    };
                    let _ = tray.set_tooltip(Some(tooltip));
                }
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
