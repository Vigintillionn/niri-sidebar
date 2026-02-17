use crate::commands::movefrom::move_to;
use crate::commands::reorder;
use crate::commands::togglewindow::add_to_sidebar;
use crate::config::load_config;
use crate::niri::connect;
use crate::state::{get_default_cache_dir, load_state, save_state};
use crate::window_rules::resolve_auto_add;
use crate::{Ctx, NiriClient};
use anyhow::Result;
use fslock::LockFile;
use niri_ipc::socket::Socket;
use niri_ipc::{Event, Request, Window};

pub fn listen(mut ctx: Ctx<Socket>) -> Result<()> {
    let _ = ctx.socket.send(Request::EventStream)?;
    let mut read_event = ctx.socket.read_events();
    println!("niri-sidebar: Listening for window events...");

    while let Ok(event) = read_event() {
        match event {
            Event::WindowClosed { id } => handle_close_event(id)?,
            Event::WindowFocusChanged { .. } => handle_focus_change()?,
            Event::WorkspaceActivated { id, focused: true } => handle_workspace_focus(id)?,
            Event::WindowOpenedOrChanged { window } => handle_new_window(&window)?,
            _ => {}
        }
    }

    Ok(())
}

fn get_ctx() -> Result<(Ctx<Socket>, LockFile)> {
    let cache_dir = get_default_cache_dir()?;
    let mut lock_path = cache_dir.clone();
    lock_path.push("instance.lock");
    let mut lock_file = LockFile::open(&lock_path)?;
    lock_file.lock()?;

    let state = load_state(&cache_dir)?;
    let config = load_config();
    let ctx = Ctx {
        state,
        config,
        socket: connect()?,
        cache_dir,
    };

    Ok((ctx, lock_file))
}

fn handle_close_event(closed_id: u64) -> Result<()> {
    let (mut ctx, _lock) = get_ctx()?;
    process_close(&mut ctx, closed_id)
}

fn handle_focus_change() -> Result<()> {
    let (mut ctx, _lock) = get_ctx()?;
    process_focus(&mut ctx)
}

fn handle_workspace_focus(ws_id: u64) -> Result<()> {
    let (mut ctx, _lock) = get_ctx()?;
    if ctx.config.interaction.sticky {
        process_move(&mut ctx, ws_id)
    } else {
        Ok(())
    }
}

fn handle_new_window(window: &Window) -> Result<()> {
    let (mut ctx, _lock) = get_ctx()?;
    process_new_window(&mut ctx, window)
}

pub fn process_close<C: NiriClient>(ctx: &mut Ctx<C>, closed_id: u64) -> Result<()> {
    if let Some(index) = ctx
        .state
        .windows
        .iter()
        .position(|(id, _, _)| *id == closed_id)
    {
        println!("Sidebar window {} closed. Reordering...", closed_id);

        ctx.state.windows.remove(index);
        save_state(&ctx.state, &ctx.cache_dir)?;
        dbg!(&ctx.state);

        reorder(ctx)?;
    }

    Ok(())
}

pub fn process_focus<C: NiriClient>(ctx: &mut Ctx<C>) -> Result<()> {
    reorder(ctx)?;
    Ok(())
}

pub fn process_move<C: NiriClient>(ctx: &mut Ctx<C>, ws_id: u64) -> Result<()> {
    let windows: Vec<_> = ctx.socket.get_windows()?;
    let sidebar_windows = windows
        .iter()
        .filter(|w| ctx.state.windows.iter().any(|&(id, _, _)| id == w.id))
        .collect();
    move_to(ctx, sidebar_windows, ws_id)?;
    Ok(())
}

pub fn process_new_window<C: NiriClient>(ctx: &mut Ctx<C>, window: &Window) -> Result<()> {
    // If window is removed from sidebar a WindowOpenedOrChanged event will happen
    // and this if let will catch that and remove id from vector, prevents auto_add
    // from being triggered immediately after window is removed from sidebar
    if let Some(index) = ctx
        .state
        .ignored_windows
        .iter()
        .position(|id| id == &window.id)
    {
        ctx.state.ignored_windows.remove(index);
        return Ok(());
    }

    if resolve_auto_add(&ctx.config.window_rule, window)
        && !ctx.state.windows.iter().any(|(id, _, _)| *id == window.id)
    {
        // NOTE: for now using add_to_sidebar, probably better than relying on new window being
        // focused, niri window rules will interfere with that
        add_to_sidebar(ctx, window)?;
        save_state(&ctx.state, &ctx.cache_dir)?;
        reorder(ctx)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, WindowRule};
    use crate::state::AppState;
    use crate::test_utils::{MockNiri, mock_window};
    use niri_ipc::{Action, WorkspaceReferenceArg};
    use regex::Regex;
    use tempfile::tempdir;

    #[test]
    fn test_process_close_removes_window_and_reorders() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("NIRI_SIDEBAR_TEST_DIR", temp_dir.path());
        }

        let mut state = AppState::default();
        state.windows.push((100, 500, 500));
        state.windows.push((200, 500, 500));

        let w100 = mock_window(100, true, true, 1);
        let w200 = mock_window(200, true, true, 1);
        let mock = MockNiri::new(vec![w100, w200]);

        let mut ctx = Ctx {
            state,
            config: Config::default(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        process_close(&mut ctx, 100).expect("Process close failed");

        // 100 removed
        assert!(!ctx.state.windows.iter().any(|(id, _, _)| *id == 100));
        assert_eq!(ctx.state.windows.len(), 1);
        assert_eq!(ctx.state.windows[0].0, 200);
        // Reorder should have run (sending actions)
        assert!(!ctx.socket.sent_actions.is_empty());
    }

    #[test]
    fn test_process_close_ignores_unknown_window() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("NIRI_SIDEBAR_TEST_DIR", temp_dir.path());
        }

        let mut state = AppState::default();
        state.windows.push((100, 500, 500));

        let mock = MockNiri::new(vec![]);

        let mut ctx = Ctx {
            state,
            config: Config::default(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        process_close(&mut ctx, 999).expect("Process close failed");

        // State should still have Window 100
        assert_eq!(ctx.state.windows.len(), 1);
        assert_eq!(ctx.state.windows[0].0, 100);

        // No reorder actions should have been sent
        assert!(ctx.socket.sent_actions.is_empty());
    }

    #[test]
    fn test_process_move_consolidates_tracked_windows_from_all_workspaces() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("NIRI_SIDEBAR_TEST_DIR", temp_dir.path());
        }

        let mut state = AppState::default();
        state.windows.push((10, 100, 100));
        state.windows.push((20, 100, 100));

        // Window 10: Tracked, on WS 1
        let w10 = mock_window(10, true, false, 1);
        // Window 20: Tracked, on WS 2
        let w20 = mock_window(20, true, false, 2);
        // Window 30: Untracked, on WS 1
        let w30 = mock_window(30, true, false, 1);

        let mock = MockNiri::new(vec![w10, w20, w30]);

        let mut ctx = Ctx {
            state,
            config: Config::default(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        let target_ws = 99;

        process_move(&mut ctx, target_ws).expect("process_move failed");
        let actions = &ctx.socket.sent_actions;

        // Should have 2 actions (for ID 10 and 20)
        assert_eq!(actions.len(), 2);

        let check_action = |act: &Action, expected_id: u64| {
            if let Action::MoveWindowToWorkspace {
                window_id,
                reference,
                ..
            } = act
            {
                assert_eq!(*window_id, Some(expected_id));
                match reference {
                    WorkspaceReferenceArg::Id(id) => assert_eq!(*id, target_ws),
                    _ => panic!("Wrong target workspace"),
                }
            } else {
                panic!("Wrong action type");
            }
        };

        check_action(&actions[0], 10);
        check_action(&actions[1], 20);
    }

    #[test]
    fn test_process_new_window_adds_when_autoadd_true() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("NIRI_SIDEBAR_TEST_DIR", temp_dir.path());
        }

        let state = AppState::default();

        let w100 = mock_window(100, true, true, 1);
        let mock = MockNiri::new(vec![w100]);

        let config = Config {
            window_rule: vec![WindowRule {
                app_id: Some(Regex::new(r"test").unwrap()),
                auto_add: true,
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut ctx = Ctx {
            state,
            config,
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        let w100 = mock_window(100, true, true, 1);
        process_new_window(&mut ctx, &w100).expect("Process new window failed");

        // 100 added
        assert!(ctx.state.windows.iter().any(|(id, _, _)| *id == 100));
        assert_eq!(ctx.state.windows.len(), 1);
        assert_eq!(ctx.state.windows[0].0, 100);
        // Reorder should have run (sending actions)
        assert!(!ctx.socket.sent_actions.is_empty());
    }

    #[test]
    fn test_process_new_window_ignores_when_autoadd_false() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("NIRI_SIDEBAR_TEST_DIR", temp_dir.path());
        }

        let state = AppState::default();

        let w100 = mock_window(100, true, true, 1);
        let mock = MockNiri::new(vec![w100]);

        let config = Config {
            window_rule: vec![WindowRule {
                app_id: Some(Regex::new(r"test").unwrap()),
                auto_add: false,
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut ctx = Ctx {
            state,
            config,
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        let w100 = mock_window(100, true, true, 1);
        process_new_window(&mut ctx, &w100).expect("Process new window failed");

        // 100 ignored
        assert!(!ctx.state.windows.iter().any(|(id, _, _)| *id == 100));
        assert_eq!(ctx.state.windows.len(), 0);
        // Reorder should have run (sending actions)
        assert!(ctx.socket.sent_actions.is_empty());
    }

    #[test]
    fn test_process_new_window_ignores_when_no_rule() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            std::env::set_var("NIRI_SIDEBAR_TEST_DIR", temp_dir.path());
        }

        let state = AppState::default();

        let w100 = mock_window(100, true, true, 1);
        let mock = MockNiri::new(vec![w100]);

        let mut ctx = Ctx {
            state,
            config: Config::default(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        let w100 = mock_window(100, true, true, 1);
        process_new_window(&mut ctx, &w100).expect("Process new window failed");

        // 100 ignored
        assert!(!ctx.state.windows.iter().any(|(id, _, _)| *id == 100));
        assert_eq!(ctx.state.windows.len(), 0);
        // Reorder should have run (sending actions)
        assert!(ctx.socket.sent_actions.is_empty());
    }
}
