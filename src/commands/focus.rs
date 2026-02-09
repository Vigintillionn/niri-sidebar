use crate::niri;
use crate::Ctx;
use anyhow::{bail, Result};
use niri_ipc::{Action, Request};

#[derive(Clone, Copy)]
pub enum Direction {
    Next,
    Prev,
}

pub fn focus_cycle(ctx: &mut Ctx, direction: Direction) -> Result<()> {
    let current_ws = niri::get_active_workspace_id(&mut ctx.socket)?;
    let all_windows = niri::get_windows(&mut ctx.socket)?;

    let sidebar_ids: Vec<u64> = ctx.state.windows.iter().map(|(id, _, _)| *id).collect();
    let mut sidebar_windows: Vec<_> = all_windows
        .iter()
        .filter(|w| {
            w.is_floating && w.workspace_id == Some(current_ws) && sidebar_ids.contains(&w.id)
        })
        .collect();

    if sidebar_windows.is_empty() {
        bail!("No sidebar windows on the current workspace");
    }

    // Sort by state order for stable ordering
    sidebar_windows.sort_by_key(|w| {
        sidebar_ids
            .iter()
            .position(|id| *id == w.id)
            .unwrap_or(usize::MAX)
    });
    if ctx.state.is_flipped {
        sidebar_windows.reverse();
    }

    let focused_idx = sidebar_windows.iter().position(|w| w.is_focused);

    let target_idx = match focused_idx {
        Some(idx) => {
            let len = sidebar_windows.len();
            match direction {
                Direction::Next => (idx + 1) % len,
                Direction::Prev => (idx + len - 1) % len,
            }
        }
        // No sidebar window focused: enter from the matching end
        None => match direction {
            Direction::Next => 0,
            Direction::Prev => sidebar_windows.len() - 1,
        },
    };

    let target_id = sidebar_windows[target_idx].id;
    let action = Action::FocusWindow { id: target_id };
    let _ = ctx.socket.send(Request::Action(action))?;

    Ok(())
}
