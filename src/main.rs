use anyhow::Result;
use clap::{Parser, Subcommand};
use fslock::LockFile;
use niri_sidebar::config::load_config;
use niri_sidebar::state::{get_default_cache_dir, load_state};
use niri_sidebar::{AppState, Ctx, config, niri::connect};
use niri_sidebar::{Direction, commands};

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
    /// Focus and cycle through the windows in the sidebar
    Focus {
        #[arg(value_enum, default_value_t = Direction::Next)]
        direction: Direction,
    },
    /// Move the sidebar from a specific workspace to the current workspace
    MoveFrom {
        #[arg()]
        workspace: u64,
    },
    /// Generate a default config file if none exists
    Init,
    /// Run a daemon to listen for window close events
    Listen,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Init doesn't require locks or state loading
    if let Commands::Init = cli.command {
        return config::init_config();
    }

    let cache_dir = get_default_cache_dir()?;
    let mut lock_path = cache_dir.clone();
    lock_path.push("instance.lock");
    let mut lock_file = LockFile::open(&lock_path)?;

    // Listener will handle its own locking when it needs to write
    if !matches!(cli.command, Commands::Listen) && !lock_file.try_lock()? {
        lock_file.lock()?;
    }
    let config = load_config();
    // Listener will load state on demand
    let state = if matches!(cli.command, Commands::Listen) {
        AppState::default()
    } else {
        load_state(&cache_dir)?
    };
    let socket = connect()?;

    let mut ctx = Ctx {
        state,
        config,
        socket,
        cache_dir,
    };

    match cli.command {
        Commands::ToggleWindow => commands::toggle_window(&mut ctx)?,
        Commands::ToggleVisibility => commands::toggle_visibility(&mut ctx)?,
        Commands::Flip => commands::toggle_flip(&mut ctx)?,
        Commands::Reorder => commands::reorder(&mut ctx)?,
        Commands::Close => commands::close(&mut ctx)?,
        Commands::Focus { direction } => commands::focus(&mut ctx, direction)?,
        Commands::MoveFrom { workspace } => commands::move_from(&mut ctx, workspace)?,
        Commands::Init => unreachable!(),
        Commands::Listen => commands::listen(ctx)?,
    }

    Ok(())
}
