use crate::Ctx;
use crate::commands::reorder;
use crate::niri::NiriClient;
use crate::state::save_state;
use anyhow::Result;
use niri_ipc::Action;

pub fn toggle_visibility<C: NiriClient>(ctx: &mut Ctx<C>) -> Result<()> {
    let was_hidden = ctx.state.is_hidden;
    ctx.state.is_hidden = !was_hidden;
    save_state(&ctx.state, &ctx.cache_dir)?;
    reorder(ctx)?;

    if was_hidden && ctx.config.interaction.focus_on_unhide {
        let _ = ctx.socket.send_action(Action::FocusFloating {});
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, WindowState};
    use crate::test_utils::{MockNiri, mock_config, mock_window};
    use niri_ipc::{Action, PositionChange};
    use tempfile::tempdir;

    #[test]
    fn test_toggle_visibility() {
        let temp_dir = tempdir().unwrap();
        let win = mock_window(100, false, true, 1, Some((1.0, 2.0)));
        let mock = MockNiri::new(vec![win]);

        let mut state = AppState::default();
        let w1 = WindowState {
            id: 100,
            width: 300,
            height: 500,
            is_floating: false,
            position: None,
        };
        state.windows.push(w1);
        state.is_hidden = false;

        let mut ctx = Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        toggle_visibility(&mut ctx).expect("Toggle visibility failed");

        // State changed and Move action sent (Moved to Hidden X)
        assert!(ctx.state.is_hidden);

        // Screen width 1920. Peek is 10 (focused). Target X should be 1920 - 10 = 1910.
        let actions = &ctx.socket.sent_actions;
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::MoveFloatingWindow {
                id: Some(100),
                x: PositionChange::SetFixed(1910.0),
                ..
            }
        )));

        ctx.socket.sent_actions.clear();
        toggle_visibility(&mut ctx).expect("Toggle visibility failed");

        // State changed back and Move action sent (Moved to Visible X)
        assert!(!ctx.state.is_hidden);
        // Visible X = 1920 - 300 (width) - 20 (margin) = 1600
        let actions = &ctx.socket.sent_actions;
        dbg!(actions);
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::MoveFloatingWindow {
                id: Some(100),
                x: PositionChange::SetFixed(1600.0),
                ..
            }
        )));
    }
}
