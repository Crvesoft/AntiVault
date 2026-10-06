use tauri::Manager;

pub mod commands;
pub mod error;
pub mod storage;
pub mod tray;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = storage::account::get_database_path();
    let db = storage::account::Database::new(&db_path)
        .expect("Failed to initialize AntiVault SQLite database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(db)
        .setup(|app| {
            // Setup system tray
            if let Err(e) = tray::setup_tray(app.handle()) {
                tracing::error!("Failed to setup system tray: {}", e);
            }

            if let Some(win) = app.get_webview_window("main") {
                // 1. Restore saved size & state before window is shown
                if let Some(saved) = utils::window_state::load_window_state() {
                    if saved.width >= 480.0 && saved.height >= 600.0 {
                        let _ = win.set_size(tauri::LogicalSize::new(saved.width, saved.height));
                    }
                    if saved.is_maximized {
                        let _ = win.maximize();
                    }
                }

                // 2. Native OS window event listener: persist window size and intercept close to hide to tray
                let win_clone = win.clone();
                win.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            if !tray::is_quitting() && tray::get_close_to_tray() {
                                api.prevent_close();
                                let _ = win_clone.hide();
                            }
                        }
                        tauri::WindowEvent::Resized(_) => {
                            if let Ok(is_max) = win_clone.is_maximized() {
                                if !is_max {
                                    if let Ok(scale) = win_clone.scale_factor() {
                                        if let Ok(size) = win_clone.outer_size() {
                                            let w = (size.width as f64) / scale;
                                            let h = (size.height as f64) / scale;
                                            utils::window_state::save_window_state(w, h, false);
                                        }
                                    }
                                } else {
                                    utils::window_state::save_window_state(0.0, 0.0, true);
                                }
                            }
                        }
                        _ => {}
                    }
                });

                // 3. Fallback timer: ensure window is shown after 800ms if frontend has not called show()
                let win_for_timer = win.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    let _ = win_for_timer.show();
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Account commands
            commands::account::list_accounts,
            commands::account::get_account,
            commands::account::get_current_account,
            commands::account::add_account,
            commands::account::delete_account,
            commands::account::set_current_account,
            commands::account::reorder_accounts,
            commands::account::import_local_accounts,
            // Quota commands
            commands::quota::refresh_quota,
            commands::quota::get_quotas,
            commands::quota::refresh_all_quotas,
            // System integration commands
            commands::system::get_antigravity_status,
            commands::system::close_antigravity_process,
            commands::system::launch_antigravity_app,
            commands::system::switch_account,
            commands::system::save_window_size,
            commands::system::get_saved_window_size,
            commands::system::get_app_settings,
            commands::system::save_app_settings,
            commands::system::check_for_updates,
            // OAuth commands
            commands::oauth::start_google_login,
            commands::oauth::complete_google_login,
            commands::oauth::cancel_google_login,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AntiVault");
}
