use crate::commands::reorder;
use crate::niri::NiriClient;
use crate::state::save_state;
use crate::{Ctx, state::WindowState};
use anyhow::{Context, Result};
use niri_ipc::{Action, SizeChange, Window};

// TODO: restore window pos if floating
// TODO: adjust tests to check new fields

pub fn toggle_window<C: NiriClient>(ctx: &mut Ctx<C>) -> Result<()> {
    let windows = ctx.socket.get_windows()?;

    let focused = windows
        .iter()
        .find(|w| w.is_focused)
        .context("No window focused")?;

    let is_tracked = ctx.state.windows.iter().any(|w| w.id == focused.id);

    if is_tracked {
        remove_from_sidebar(ctx, focused)?;
    } else {
        add_to_sidebar(ctx, focused)?;
    }

    save_state(&ctx.state, &ctx.cache_dir)?;
    reorder(ctx)?;

    Ok(())
}

fn add_to_sidebar<C: NiriClient>(ctx: &mut Ctx<C>, window: &Window) -> Result<()> {
    let (width, height) = window.layout.window_size;
    let w_state = WindowState {
        id: window.id,
        width,
        height,
        is_floating: window.is_floating,
        position: window.layout.tile_pos_in_workspace_view,
    };
    ctx.state.windows.push(w_state);

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

fn remove_from_sidebar<C: NiriClient>(ctx: &mut Ctx<C>, window: &Window) -> Result<()> {
    let index = ctx
        .state
        .windows
        .iter()
        .position(|w| w.id == window.id)
        .context("Window was not found in sidebar state")?;
    let w_state = ctx.state.windows.remove(index);

    let _ = ctx.socket.send_action(Action::SetWindowWidth {
        change: SizeChange::SetFixed(w_state.width),
        id: Some(window.id),
    });

    let _ = ctx.socket.send_action(Action::SetWindowHeight {
        change: SizeChange::SetFixed(w_state.height),
        id: Some(window.id),
    });

    if window.is_floating {
        let _ = ctx.socket.send_action(Action::ToggleWindowFloating {
            id: Some(window.id),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::config::Config;
    use crate::state::AppState;
    use crate::test_utils::{MockNiri, mock_config, mock_window};

    // TODO: add case when floating and not when logic implemented
    #[test]
    fn test_add_to_sidebar() {
        let temp_dir = tempdir().unwrap();
        let win = mock_window(100, true, false, 1, None);
        let mock = MockNiri::new(vec![win]);

        let config = mock_config();

        let mut ctx = Ctx {
            state: AppState::default(),
            config,
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        toggle_window(&mut ctx).expect("Command failed");

        // Window 100 should be in the sidebar list with original size (1000x800)
        assert_eq!(ctx.state.windows.len(), 1);
        let w_state = &ctx.state.windows[0];
        assert_eq!(w_state.id, 100);
        assert_eq!(w_state.width, 1000);
        assert_eq!(w_state.height, 800);

        let actions = &ctx.socket.sent_actions;

        // Should toggle floating
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, Action::ToggleWindowFloating { id: Some(100) }))
        );

        // Should set width to 300 (config width)
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::SetWindowWidth {
                change: SizeChange::SetFixed(300),
                id: Some(100)
            }
        )));
    }

    #[test]
    fn test_remove_from_sidebar() {
        let temp_dir = tempdir().unwrap();
        let win = mock_window(100, true, true, 1, None);
        let mock = MockNiri::new(vec![win]);

        let mut state = AppState::default();
        let w1 = WindowState {
            id: 100,
            width: 1000,
            height: 800,
            is_floating: true,
            position: None,
        };
        state.windows.push(w1);

        let mut ctx = Ctx {
            state,
            config: Config::default(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        toggle_window(&mut ctx).expect("Command failed");

        // Should be empty now
        assert!(ctx.state.windows.is_empty());

        // Should restore original size
        let actions = &ctx.socket.sent_actions;

        // Should restore width to 1000
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::SetWindowWidth {
                change: SizeChange::SetFixed(1000),
                id: Some(100)
            }
        )));

        // Should restore height to 800
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::SetWindowHeight {
                change: SizeChange::SetFixed(800),
                id: Some(100)
            }
        )));
    }
}
