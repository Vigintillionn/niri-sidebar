mod commands;
mod config;
mod niri;
mod state;

use anyhow::Result;
use clap::{Parser, Subcommand};
use fslock::LockFile;
use niri_ipc::socket::Socket;

use crate::config::Config;
use crate::state::AppState;

#[derive(Parser)]
#[command(name = "niri-sidebar")]
#[command(about = "A floating sidebar manager for Niri")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Toggle the focused window in/out of the sidebar
    ToggleWindow,
    /// Hide or show the sidebar
    ToggleVisibility,
    /// Reverse the order of windows in the stack
    Flip,
    /// Force re-stacking of windows
    Reorder,
    /// Close the focused window and reorder the sidebar
    Close,
    /// Focus the next sidebar window
    FocusNext,
    /// Focus the previous sidebar window
    FocusPrev,
    /// Generate a default config file if none exists
    Init,
    /// Run a daemon to listen for window close events
    Listen,
}

pub struct Ctx {
    pub state: AppState,
    pub config: Config,
    pub socket: Socket,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Init doesn't require locks or state loading
    if let Commands::Init = cli.command {
        return config::init_config();
    }

    let mut lock_path = state::get_cache_dir()?;
    lock_path.push("instance.lock");
    let mut lock_file = LockFile::open(&lock_path)?;

    // Listener will handle its own locking when it needs to write
    if !matches!(cli.command, Commands::Listen) && !lock_file.try_lock()? {
        lock_file.lock()?;
    }
    let config = config::load_config();
    // Listener will load state on demand
    let state = if matches!(cli.command, Commands::Listen) {
        AppState::default()
    } else {
        state::load_state()?
    };
    let socket = niri::connect()?;

    let mut ctx = Ctx {
        state,
        config,
        socket,
    };

    match cli.command {
        Commands::ToggleWindow => commands::toggle_window(&mut ctx)?,
        Commands::ToggleVisibility => commands::toggle_visibility(&mut ctx)?,
        Commands::Flip => commands::toggle_flip(&mut ctx)?,
        Commands::Reorder => commands::reorder(&mut ctx)?,
        Commands::Close => commands::close(&mut ctx)?,
        Commands::FocusNext => commands::focus_cycle(&mut ctx, commands::Direction::Next)?,
        Commands::FocusPrev => commands::focus_cycle(&mut ctx, commands::Direction::Prev)?,
        Commands::Init => unreachable!(),
        Commands::Listen => commands::listen(ctx)?,
    }

    Ok(())
}
