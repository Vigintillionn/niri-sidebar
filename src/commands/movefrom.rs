use crate::{Ctx, NiriClient};
use anyhow::Result;
use niri_ipc::{Action, Window, WorkspaceReferenceArg};

pub fn move_from<C: NiriClient>(ctx: &mut Ctx<C>, workspace: u64) -> Result<()> {
    let active_workspace = ctx.socket.get_active_workspace()?.id;
    let windows = ctx.socket.get_windows()?;

    let windows_on_ws: Vec<_> = windows
        .iter()
        .filter(|w| {
            w.workspace_id == Some(workspace) && ctx.state.windows.iter().any(|ws| ws.id == w.id)
        })
        .collect();

    move_to(ctx, windows_on_ws, active_workspace)?;

    Ok(())
}

pub fn move_to<C: NiriClient>(ctx: &mut Ctx<C>, windows: Vec<&Window>, to_ws: u64) -> Result<()> {
    for w in windows {
        ctx.socket.send_action(Action::MoveWindowToWorkspace {
            window_id: Some(w.id),
            reference: WorkspaceReferenceArg::Id(to_ws),
            focus: false,
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::state::{AppState, WindowState};
    use crate::test_utils::{MockNiri, mock_window};
    use niri_ipc::{Action, WorkspaceReferenceArg};
    use tempfile::tempdir;

    #[test]
    fn test_move_from_only_moves_tracked_windows_on_source_workspace() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("NIRI_SIDEBAR_TEST_DIR", temp_dir.path());
        }

        // We track ID 100 and ID 300. We do NOT track ID 200.
        let mut state = AppState::default();
        let w1 = WindowState {
            id: 100,
            width: 500,
            height: 500,
            is_floating: true,
            position: Some((1.0, 2.0)),
        };
        let w2 = WindowState {
            id: 500,
            width: 500,
            height: 500,
            is_floating: true,
            position: Some((1.0, 2.0)),
        };
        state.windows.push(w1);
        state.windows.push(w2);

        // 2. Setup Mock Windows
        let source_ws = 2;
        let target_ws = 1;

        // Window 100: On Source WS (1) + Tracked in State -> SHOULD MOVE
        let w100 = mock_window(100, true, false, source_ws, Some((1.0, 2.0)));

        // Window 200: On Source WS (1) + NOT Tracked -> SHOULD IGNORE
        let w200 = mock_window(200, true, false, source_ws, Some((1.0, 2.0)));

        // Window 300: On Wrong WS (99) + Tracked -> SHOULD IGNORE
        let w300 = mock_window(300, true, false, 99, Some((1.0, 2.0)));

        // Window 400: On Target WS (2) + Focused
        let w400 = mock_window(400, true, true, target_ws, Some((1.0, 2.0)));

        let mock = MockNiri::new(vec![w100, w200, w300, w400]);

        let mut ctx = Ctx {
            state,
            config: Config::default(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        move_from(&mut ctx, source_ws).expect("move_from failed");
        let actions = &ctx.socket.sent_actions;

        // We expect exactly ONE action (for w100)
        assert_eq!(actions.len(), 1, "Should only generate 1 move action");

        if let Action::MoveWindowToWorkspace {
            window_id,
            reference,
            ..
        } = &actions[0]
        {
            assert_eq!(*window_id, Some(100), "Should move window 100");

            match reference {
                WorkspaceReferenceArg::Id(id) => {
                    assert_eq!(*id, target_ws, "Should move to active workspace (2)")
                }
                _ => panic!("Expected ID reference"),
            }
        } else {
            panic!("Unexpected action type sent to socket");
        }
    }
}
