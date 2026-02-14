use crate::niri::NiriClient;
use crate::state::WindowState;
use crate::{Ctx, Direction};
use anyhow::Result;
use niri_ipc::Action;

pub fn focus<C: NiriClient>(ctx: &mut Ctx<C>, direction: Direction) -> Result<()> {
    let len = ctx.state.windows.len();

    if len == 0 {
        return Ok(());
    }

    let active_window = ctx.socket.get_active_window()?.id;
    let current_index_opt = ctx.state.windows.iter().position(|w| w.id == active_window);

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

    if let Some(WindowState { id, .. }) = ctx.state.windows.get(next_index) {
        let _ = ctx.socket.send_action(Action::FocusWindow { id: *id });
    }

    Ok(())
}

#[cfg(test)]
mod tests_focus {
    use super::*;
    use crate::Direction;
    use crate::state::AppState;
    use crate::test_utils::{MockNiri, mock_config, mock_window};
    use niri_ipc::Action;
    use tempfile::tempdir;

    #[test]
    fn test_cycle_focus_next() {
        let temp_dir = tempdir().unwrap();
        // Sidebar has [A, B, C]. Focused is B (Index 1).
        // Next => (i + len - 1) % len => (1 + 3 - 1) % 3 = 0. So focus A

        let win_a = mock_window(1, false, true, 1, Some((1.0, 2.0)));
        let win_b = mock_window(2, true, true, 1, Some((1.0, 2.0)));
        let win_c = mock_window(3, false, true, 1, Some((1.0, 2.0)));
        let mock = MockNiri::new(vec![win_a, win_b, win_c]);

        let mut state = AppState::default();
        let w1 = WindowState {
            id: 1,
            width: 100,
            height: 100,
            is_floating: false,
            position: None,
        };
        let w2 = WindowState {
            id: 2,
            width: 100,
            height: 100,
            is_floating: true,
            position: Some((1.0, 2.0)),
        };
        let w3 = WindowState {
            id: 3,
            width: 100,
            height: 100,
            is_floating: true,
            position: Some((1.0, 2.0)),
        };
        state.windows.push(w1);
        state.windows.push(w2);
        state.windows.push(w3);

        let mut ctx = Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        focus(&mut ctx, Direction::Next).unwrap();

        assert!(
            ctx.socket
                .sent_actions
                .iter()
                .any(|a| matches!(a, Action::FocusWindow { id: 1 }))
        );
    }

    #[test]
    fn test_cycle_focus_prev() {
        let temp_dir = tempdir().unwrap();
        // Sidebar has [A, B, C]. Focused is B (Index 1).
        // Prev => (i + 1) % len => (1 + 1) % 3 = 2. So focus C
        //
        let win_a = mock_window(1, false, true, 1, Some((1.0, 2.0)));
        let win_b = mock_window(2, true, true, 1, Some((1.0, 2.0)));
        let win_c = mock_window(3, false, true, 1, Some((1.0, 2.0)));
        let mock = MockNiri::new(vec![win_a, win_b, win_c]);

        let mut state = AppState::default();
        let w1 = WindowState {
            id: 1,
            width: 100,
            height: 100,
            is_floating: false,
            position: None,
        };
        let w2 = WindowState {
            id: 2,
            width: 100,
            height: 100,
            is_floating: true,
            position: Some((1.0, 2.0)),
        };
        let w3 = WindowState {
            id: 3,
            width: 100,
            height: 100,
            is_floating: true,
            position: Some((1.0, 2.0)),
        };
        state.windows.push(w1);
        state.windows.push(w2);
        state.windows.push(w3);

        let mut ctx = Ctx {
            state,
            config: Default::default(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        focus(&mut ctx, Direction::Prev).unwrap();

        assert!(
            ctx.socket
                .sent_actions
                .iter()
                .any(|a| matches!(a, Action::FocusWindow { id: 3 }))
        );
    }

    #[test]
    fn test_enter_focus_from_outside() {
        let temp_dir = tempdir().unwrap();
        // Focused window is Z (99), not in sidebar.
        // Next: len - 1 (Last item).
        // Prev: 0 (First item).

        let win_a = mock_window(1, false, true, 1, Some((1.0, 2.0)));
        let win_b = mock_window(2, false, true, 1, Some((1.0, 2.0)));
        let win_z = mock_window(99, true, false, 1, None);
        let mock = MockNiri::new(vec![win_a, win_b, win_z]);

        let mut state = AppState::default();
        let w1 = WindowState {
            id: 1,
            width: 100,
            height: 100,
            is_floating: false,
            position: None,
        };
        let w2 = WindowState {
            id: 2,
            width: 100,
            height: 100,
            is_floating: true,
            position: Some((1.0, 2.0)),
        };
        state.windows.push(w1);
        state.windows.push(w2);

        let mut ctx = Ctx {
            state,
            config: Default::default(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        // Next (Should focus last item -> B)
        focus(&mut ctx, Direction::Next).unwrap();
        assert!(
            ctx.socket
                .sent_actions
                .iter()
                .any(|a| matches!(a, Action::FocusWindow { id: 2 }))
        );

        ctx.socket.sent_actions.clear();

        // Prev (Should focus first item -> A)
        focus(&mut ctx, Direction::Prev).unwrap();
        assert!(
            ctx.socket
                .sent_actions
                .iter()
                .any(|a| matches!(a, Action::FocusWindow { id: 1 }))
        );
    }

    #[test]
    fn test_focus_empty_sidebar() {
        let temp_dir = tempdir().unwrap();
        let win = mock_window(99, true, false, 1, None);
        let mock = MockNiri::new(vec![win]);

        // Empty state
        let mut ctx = Ctx {
            state: AppState::default(),
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        focus(&mut ctx, Direction::Next).unwrap();
        assert!(ctx.socket.sent_actions.is_empty());
        focus(&mut ctx, Direction::Prev).unwrap();
        assert!(ctx.socket.sent_actions.is_empty());
    }
}
