use crate::Ctx;
use crate::commands::reorder;
use crate::niri::get_windows;
use crate::state::save_state;
use anyhow::{Context, Result};
use niri_ipc::{Action, Request, SizeChange, Window};

pub fn toggle_window(ctx: &mut Ctx) -> Result<()> {
    let windows = get_windows(&mut ctx.socket)?;

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

fn add_to_sidebar(ctx: &mut Ctx, window: &Window) -> Result<()> {
    let (width, height) = window.layout.window_size;
    ctx.state.windows.push((window.id, width, height));

    if !window.is_floating {
        let action = Action::ToggleWindowFloating {
            id: Some(window.id),
        };
        let _ = ctx.socket.send(Request::Action(action))?;
    }

    let set_w = Action::SetWindowWidth {
        change: SizeChange::SetFixed(ctx.config.sidebar_width),
        id: Some(window.id),
    };
    let _ = ctx.socket.send(Request::Action(set_w))?;

    let set_h = Action::SetWindowHeight {
        change: SizeChange::SetFixed(ctx.config.sidebar_height),
        id: Some(window.id),
    };
    let _ = ctx.socket.send(Request::Action(set_h))?;

    Ok(())
}

fn remove_from_sidebar(ctx: &mut Ctx, window: &Window) -> Result<()> {
    let index = ctx
        .state
        .windows
        .iter()
        .position(|(id, _, _)| *id == window.id)
        .context("Window was not found in sidebar state")?;
    let (_, orig_w, orig_h) = ctx.state.windows.remove(index);

    let set_w = Action::SetWindowWidth {
        change: SizeChange::SetFixed(orig_w),
        id: Some(window.id),
    };
    let _ = ctx.socket.send(Request::Action(set_w))?;

    let set_h = Action::SetWindowHeight {
        change: SizeChange::SetFixed(orig_h),
        id: Some(window.id),
    };
    let _ = ctx.socket.send(Request::Action(set_h))?;

    if window.is_floating {
        let toggle_float = Action::ToggleWindowFloating {
            id: Some(window.id),
        };
        let _ = ctx.socket.send(Request::Action(toggle_float))?;
    }

    // Return focus to the tiling layout so keyboard navigation works immediately
    let _ = ctx.socket.send(Request::Action(Action::FocusTiling {}))?;

    Ok(())
}
