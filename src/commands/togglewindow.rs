use crate::Ctx;
use crate::commands::reorder;
use crate::niri::NiriClient;
use crate::state::save_state;
use crate::window_rules::resolve_window_size;
use anyhow::{Context, Result};
use niri_ipc::{Action, SizeChange, Window};

pub fn toggle_window<C: NiriClient>(ctx: &mut Ctx<C>) -> Result<()> {
    let focused = ctx.socket.get_active_window()?;

    let is_tracked = ctx.state.windows.iter().any(|(id, _, _)| *id == focused.id);

    if is_tracked {
        remove_from_sidebar(ctx, &focused)?;
    } else {
        add_to_sidebar(ctx, &focused)?;
    }

    save_state(&ctx.state, &ctx.cache_dir)?;
    reorder(ctx)?;

    Ok(())
}

// NOTE: wasn't pub previously, changed it for now so that listen can use this directly and pass
// Window instead of relying on toggle_window
pub fn add_to_sidebar<C: NiriClient>(ctx: &mut Ctx<C>, window: &Window) -> Result<()> {
    let (width, height) = window.layout.window_size;
    ctx.state.windows.push((window.id, width, height));

    if !window.is_floating {
        let _ = ctx.socket.send_action(Action::ToggleWindowFloating {
            id: Some(window.id),
        });
    }

    let (target_width, target_height) = resolve_window_size(
        &ctx.config.window_rule,
        window,
        ctx.config.geometry.width,
        ctx.config.geometry.height,
    );

    let _ = ctx.socket.send_action(Action::SetWindowWidth {
        change: SizeChange::SetFixed(target_width),
        id: Some(window.id),
    });

    let _ = ctx.socket.send_action(Action::SetWindowHeight {
        change: SizeChange::SetFixed(target_height),
        id: Some(window.id),
    });

    Ok(())
}

fn remove_from_sidebar<C: NiriClient>(ctx: &mut Ctx<C>, window: &Window) -> Result<()> {
    let index = ctx
        .state
        .windows
        .iter()
        .position(|(id, _, _)| *id == window.id)
        .context("Window was not found in sidebar state")?;
    let (id, orig_w, orig_h) = ctx.state.windows.remove(index);

    ctx.state.ignored_windows.push(id);

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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::config::Config;
    use crate::state::AppState;
    use crate::test_utils::{MockNiri, mock_config, mock_window};

    #[test]
    fn test_add_to_sidebar() {
        let temp_dir = tempdir().unwrap();
        let win = mock_window(100, true, false, 1);
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
        let (id, w, h) = ctx.state.windows[0];
        assert_eq!(id, 100);
        assert_eq!(w, 1000);
        assert_eq!(h, 800);

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
        let win = mock_window(100, true, true, 1);
        let mock = MockNiri::new(vec![win]);

        let mut state = AppState::default();
        state.windows.push((100, 1000, 800));

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

    #[test]
    fn test_add_to_sidebar_with_window_rule() {
        let temp_dir = tempdir().unwrap();
        // Window with specific app_id to match rule
        let mut win = mock_window(100, true, false, 1);
        win.app_id = Some("special".into());

        let mock = MockNiri::new(vec![win]);
        let mut config = mock_config();

        use crate::config::WindowRule;
        use regex::Regex;
        config.window_rule = vec![WindowRule {
            app_id: Some(Regex::new("special").unwrap()),
            width: Some(500),
            height: Some(600),
            ..Default::default()
        }];
        let mut ctx = Ctx {
            state: AppState::default(),
            config,
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };
        toggle_window(&mut ctx).expect("Command failed");

        assert_eq!(ctx.state.windows.len(), 1);

        let actions = &ctx.socket.sent_actions;
        // Should set width to 500 (Rule width), not 300 (Config default)
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::SetWindowWidth {
                change: SizeChange::SetFixed(500),
                id: Some(100)
            }
        )));

        // Should set height to 600 (Rule height)
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::SetWindowHeight {
                change: SizeChange::SetFixed(600),
                id: Some(100)
            }
        )));
    }
}
