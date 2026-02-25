use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_CONFIG_STR: &str = include_str!("../default_config.toml");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SidebarPosition {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub geometry: Geometry,
    pub margins: Margins,
    pub interaction: Interaction,
    #[serde(default)]
    pub window_rule: Vec<WindowRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Geometry {
    pub width: i32,
    pub height: i32,
    pub gap: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Margins {
    #[serde(default = "default_margin")]
    pub top: i32,
    #[serde(default = "default_margin")]
    pub right: i32,
    #[serde(default = "default_margin")]
    pub left: i32,
    #[serde(default = "default_margin")]
    pub bottom: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interaction {
    pub peek: i32,
    pub focus_peek: Option<i32>,
    #[serde(default = "default_position")]
    pub position: SidebarPosition,
    #[serde(default = "default_sticky")]
    pub sticky: bool,
    #[serde(default)]
    pub focus_on_unhide: bool,
}

impl Interaction {
    pub fn get_focus_peek(&self) -> i32 {
        self.focus_peek.unwrap_or(self.peek)
    }
}

fn default_sticky() -> bool {
    false
}

fn default_position() -> SidebarPosition {
    SidebarPosition::Right
}

fn default_margin() -> i32 {
    0
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WindowRule {
    #[serde(default, with = "serde_regex")]
    pub app_id: Option<Regex>,
    #[serde(default, with = "serde_regex")]
    pub title: Option<Regex>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub peek: Option<i32>,
    pub focus_peek: Option<i32>,
    #[serde(default)]
    pub auto_add: bool,
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG_STR).expect("Default config file is invalid TOML")
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("Could not find config directory")?;
    path.push("niri-sidebar");
    Ok(path)
}

pub fn load_config() -> Config {
    let Ok(mut path) = get_config_dir() else {
        return Config::default();
    };
    path.push("config.toml");

    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            match toml::from_str(&content) {
                Ok(cfg) => return cfg,
                Err(e) => eprintln!("Error parsing config.toml: {}. Using defaults.", e),
            }
        }
    }
    Config::default()
}

pub fn init_config() -> Result<()> {
    let mut path = get_config_dir()?;

    if !path.exists() {
        fs::create_dir_all(&path)?;
        println!("Created directory: {:?}", path);
    }

    path.push("config.toml");

    if path.exists() {
        anyhow::bail!("Config file already exists at {:?}", path);
    }

    fs::write(&path, DEFAULT_CONFIG_STR)?;
    println!("Default config written to {:?}", path);
    Ok(())
}
