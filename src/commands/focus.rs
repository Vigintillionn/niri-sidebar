use crate::niri::get_active_window;
use crate::{Ctx, Direction};
use anyhow::Result;
use niri_ipc::{Action, Request};

pub fn focus(ctx: &mut Ctx, direction: Direction) -> Result<()> {
    let len = ctx.state.windows.len();

    if len == 0 {
        return Ok(());
    }

    let active_window = get_active_window(&mut ctx.socket)?.id;
    let current_index_opt = ctx
        .state
        .windows
        .iter()
        .position(|&(id, _, _)| id == active_window);

    let next_index = if let Some(i) = current_index_opt {
        match direction {
            Direction::Next => (i + len - 1) % len,
            Direction::Prev => (i + 1) % len,
        }
    } else {
        match direction {
            Direction::Next => len - 1,
            Direction::Prev => 0,
        }
    };

    if let Some((id, _, _)) = ctx.state.windows.get(next_index) {
        let focus = Action::FocusWindow { id: *id };
        let _ = ctx.socket.send(Request::Action(focus));
    }

    Ok(())
}
