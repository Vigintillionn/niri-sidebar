pub mod commands;
pub mod config;
pub mod niri;
pub mod state;
pub mod window_rules;

use std::path::PathBuf;

use clap::ValueEnum;

pub use crate::config::Config;
pub use crate::niri::NiriClient;
pub use crate::state::AppState;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

pub struct Ctx<C: NiriClient> {
    pub state: AppState,
    pub config: Config,
    pub socket: C,
    pub cache_dir: PathBuf,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Direction {
    Next,
    Prev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowTarget {
    width: i32,
    height: i32,
}
