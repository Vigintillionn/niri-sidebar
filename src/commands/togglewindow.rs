use crate::Ctx;
use crate::commands::reorder;
use crate::niri::NiriClient;
use crate::state::save_state;
use anyhow::{Context, Result};
use niri_ipc::socket::Socket;
use niri_ipc::{Action, SizeChange, Window};

pub fn toggle_window(ctx: &mut Ctx<Socket>) -> Result<()> {
    let windows = ctx.socket.get_windows()?;

    let focused = windows
        .iter()
        .find(|w| w.is_focused)
        .context("No window focused")?;

    let is_tracked = ctx.state.windows.iter().any(|(id, _, _)| *id == focused.id);

    if is_tracked {
        remove_from_sidebar(ctx, focused)?;
    } else {
        add_to_sidebar(ctx, focused)?;
    }

    save_state(&ctx.state)?;
    reorder(ctx)?;

    Ok(())
}

fn add_to_sidebar(ctx: &mut Ctx<Socket>, window: &Window) -> Result<()> {
    let (width, height) = window.layout.window_size;
    ctx.state.windows.push((window.id, width, height));

    if !window.is_floating {
        let _ = ctx.socket.send_action(Action::ToggleWindowFloating {
            id: Some(window.id),
        });
    }

    let _ = ctx.socket.send_action(Action::SetWindowWidth {
        change: SizeChange::SetFixed(ctx.config.geometry.width),
        id: Some(window.id),
    });

    let _ = ctx.socket.send_action(Action::SetWindowHeight {
        change: SizeChange::SetFixed(ctx.config.geometry.height),
        id: Some(window.id),
    });

    Ok(())
}

fn remove_from_sidebar(ctx: &mut Ctx<Socket>, window: &Window) -> Result<()> {
    let index = ctx
        .state
        .windows
        .iter()
        .position(|(id, _, _)| *id == window.id)
        .context("Window was not found in sidebar state")?;
    let (_, orig_w, orig_h) = ctx.state.windows.remove(index);

    let _ = ctx.socket.send_action(Action::SetWindowWidth {
        change: SizeChange::SetFixed(orig_w),
        id: Some(window.id),
    });

    let _ = ctx.socket.send_action(Action::SetWindowHeight {
        change: SizeChange::SetFixed(orig_h),
        id: Some(window.id),
    });

    if window.is_floating {
        let _ = ctx.socket.send_action(Action::ToggleWindowFloating {
            id: Some(window.id),
        });
    }

    Ok(())
}
