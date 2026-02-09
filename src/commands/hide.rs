use crate::Ctx;
use crate::commands::reorder;
use crate::state::save_state;
use anyhow::Result;
use niri_ipc::socket::Socket;

pub fn toggle_visibility(ctx: &mut Ctx<Socket>) -> Result<()> {
    ctx.state.is_hidden = !ctx.state.is_hidden;
    save_state(&ctx.state)?;
    reorder(ctx)?;
    Ok(())
}
