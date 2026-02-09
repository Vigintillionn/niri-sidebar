use std::collections::HashSet;

use crate::Ctx;
use crate::niri::NiriClient;
use crate::state::save_state;
use anyhow::Result;
use niri_ipc::socket::Socket;
use niri_ipc::{Action, PositionChange, Request};

pub fn reorder(ctx: &mut Ctx<Socket>) -> Result<()> {
    let (display_w, display_h) = ctx.socket.get_screen_dimensions()?;
    let current_ws = ctx.socket.get_active_workspace()?.id;
    let all_windows = ctx.socket.get_windows()?;

    let sidebar_ids: Vec<u64> = ctx.state.windows.iter().map(|(id, _, _)| *id).collect();
    let mut sidebar_windows: Vec<_> = all_windows
        .iter()
        .filter(|w| {
            w.is_floating && w.workspace_id == Some(current_ws) && sidebar_ids.contains(&w.id)
        })
        .collect();

    let active_ids: HashSet<u64> = all_windows.iter().map(|w| w.id).collect();

    ctx.state
        .windows
        .retain(|(id, _, _)| active_ids.contains(id));
    save_state(&ctx.state)?;

    // Sort by ID for stable ordering
    sidebar_windows.sort_by_key(|w| {
        sidebar_ids
            .iter()
            .position(|id| *id == w.id)
            .unwrap_or(usize::MAX)
    });
    if ctx.state.is_flipped {
        sidebar_windows.reverse();
    }

    let sidebar_w = ctx.config.geometry.width;
    let sidebar_h = ctx.config.geometry.height;
    let gap = ctx.config.geometry.gap;
    let off_top = ctx.config.margins.top;
    let off_right = ctx.config.margins.right;
    let peek = ctx.config.interaction.peek;
    let focus_peek = ctx.config.interaction.focus_peek;

    let base_x = display_w - sidebar_w - off_right;
    let hidden_x = display_w - peek;
    let focus_hidden_x = display_w - focus_peek;

    let base_y = display_h - sidebar_h - off_top;

    for (idx, window) in sidebar_windows.iter().enumerate() {
        let target_x = if ctx.state.is_hidden {
            if window.is_focused {
                focus_hidden_x
            } else {
                hidden_x
            }
        } else {
            base_x
        };

        let stack_offset = idx as i32 * (sidebar_h + gap);
        let target_y = base_y - stack_offset;

        let action = Action::MoveFloatingWindow {
            id: Some(window.id),
            x: PositionChange::SetFixed(target_x.into()),
            y: PositionChange::SetFixed(target_y.into()),
        };

        let _ = ctx.socket.send(Request::Action(action))?;
    }
    Ok(())
}
