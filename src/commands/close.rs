use crate::Ctx;
use crate::commands::reorder;
use crate::niri::NiriClient;
use crate::state::save_state;
use anyhow::{Context, Result};
use niri_ipc::Action;
use niri_ipc::socket::Socket;

pub fn close(ctx: &mut Ctx<Socket>) -> Result<()> {
    let windows = ctx.socket.get_windows()?;
    let focused = windows
        .iter()
        .find(|w| w.is_focused)
        .context("No window focused")?;

    if let Some(index) = ctx
        .state
        .windows
        .iter()
        .position(|(id, _, _)| *id == focused.id)
    {
        ctx.state.windows.remove(index);
        save_state(&ctx.state)?;
    }

    let _ = ctx.socket.send_action(Action::CloseWindow {
        id: Some(focused.id),
    });
    reorder(ctx)?;

    Ok(())
}
