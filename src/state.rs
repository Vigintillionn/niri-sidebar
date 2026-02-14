use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct AppState {
    pub windows: Vec<WindowState>,
    #[serde(default)]
    pub ignored_windows: Vec<u64>,
    #[serde(default)]
    pub is_hidden: bool,
    #[serde(default)]
    pub is_flipped: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct WindowState {
    pub id: u64,
    pub width: i32,
    pub height: i32,
    pub is_floating: bool,
    pub position: Option<(f64, f64)>,
}

pub fn get_default_cache_dir() -> Result<PathBuf> {
    let mut path = dirs::cache_dir().context("Could not find cache directory")?;
    path.push("niri-sidebar");
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

pub fn load_state(base_dir: &Path) -> Result<AppState> {
    let mut path = base_dir.to_path_buf();
    path.push("state.json");
    if path.exists() {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    } else {
        Ok(AppState::default())
    }
}

pub fn save_state(state: &AppState, base_dir: &Path) -> Result<()> {
    let mut path = base_dir.to_path_buf();
    path.push("state.json");
    let content = serde_json::to_string_pretty(state)?;
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = tempdir().unwrap();

        let w1 = WindowState {
            id: 100,
            width: 500,
            height: 400,
            is_floating: false,
            position: None,
        };
        let w2 = WindowState {
            id: 200,
            width: 1920,
            height: 1080,
            is_floating: true,
            position: Some((1.0, 2.0)),
        };

        let original_state = AppState {
            windows: vec![w1, w2],
            ignored_windows: vec![100, 200],
            is_hidden: true,
            is_flipped: true,
        };

        save_state(&original_state, temp_dir.path()).expect("Failed to save state");
        let loaded_state = load_state(temp_dir.path()).expect("Failed to load state");

        assert_eq!(original_state, loaded_state);

        let mut expected_path = temp_dir.path().to_path_buf();
        expected_path.push("state.json");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_load_defaults_if_no_file() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("NIRI_SIDEBAR_TEST_DIR", temp_dir.path());
        }

        let state = load_state(temp_dir.path()).expect("Should not fail on missing file");
        assert_eq!(state, AppState::default());
        assert!(state.windows.is_empty());
    }

    #[test]
    fn test_handles_corrupted_json() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("NIRI_SIDEBAR_TEST_DIR", temp_dir.path());
        }

        let mut path = temp_dir.path().to_path_buf();
        path.push("state.json");
        fs::write(&path, "{ bad_json: ").unwrap();

        let state = load_state(temp_dir.path()).expect("Should recover from bad JSON");

        assert_eq!(state, AppState::default());
    }
}
