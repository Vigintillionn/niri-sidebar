use crate::Ctx;
use crate::commands::reorder;
use crate::config::Config;
use crate::niri::connect;
use crate::state::{AppState, get_cache_dir, load_state, save_state};
use anyhow::Result;
use fslock::LockFile;
use niri_ipc::socket::Socket;
use niri_ipc::{Event, Request};

pub fn listen(mut ctx: Ctx<Socket>) -> Result<()> {
    let _ = ctx.socket.send(Request::EventStream)?;
    let mut read_event = ctx.socket.read_events();
    println!("niri-sidebar: Listening for window events...");

    while let Ok(event) = read_event() {
        match event {
            Event::WindowClosed { id } => handle_close_event(id)?,
            Event::WindowFocusChanged { .. } => handle_focus_change()?,
            _ => {}
        }
    }

    Ok(())
}

fn get_ctx() -> Result<(AppState, Config)> {
    let mut lock_path = get_cache_dir()?;
    lock_path.push("instance.lock");
    let mut lock_file = LockFile::open(&lock_path)?;
    lock_file.lock()?;

    let state = load_state()?;
    let config = crate::config::load_config();

    Ok((state, config))
}

fn handle_close_event(closed_id: u64) -> Result<()> {
    let (mut state, config) = get_ctx()?;

    if let Some(index) = state.windows.iter().position(|(id, _, _)| *id == closed_id) {
        println!("Sidebar window {} closed. Reordering...", closed_id);

        state.windows.remove(index);
        save_state(&state)?;

        let action_socket = connect()?;
        let mut action_ctx = Ctx {
            state,
            config,
            socket: action_socket,
        };

        if let Err(e) = reorder(&mut action_ctx) {
            eprintln!("Failed to reorder: {}", e);
        }
    }

    Ok(())
}

fn handle_focus_change() -> Result<()> {
    let (state, config) = get_ctx()?;
    let mut ctx = Ctx {
        state,
        config,
        socket: connect()?,
    };
    reorder(&mut ctx)?;

    Ok(())
}
