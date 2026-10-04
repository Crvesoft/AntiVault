use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    pub is_maximized: bool,
}

pub fn get_window_state_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("AntiVault");
    path.push("window_state.json");
    path
}

pub fn load_window_state() -> Option<WindowState> {
    let path = get_window_state_path();
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(state) = serde_json::from_str::<WindowState>(&content) {
            return Some(state);
        }
    }
    None
}

pub fn save_window_state(width: f64, height: f64, is_maximized: bool) {
    let mut state = load_window_state().unwrap_or(WindowState {
        width: 580.0,
        height: 760.0,
        is_maximized: false,
    });

    if is_maximized {
        state.is_maximized = true;
    } else {
        if width >= 480.0 && height >= 600.0 {
            state.width = width.round();
            state.height = height.round();
        }
        state.is_maximized = false;
    }

    let path = get_window_state_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = serde_json::to_string(&state) {
        let _ = std::fs::write(path, content);
    }
}
