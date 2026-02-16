use crate::config::SidebarPosition;
use crate::niri::NiriClient;
use crate::state::save_state;
use crate::{Ctx, WindowTarget};
use anyhow::Result;
use niri_ipc::{Action, PositionChange, Window};
use std::collections::HashSet;

fn resolve_dimensions<C: NiriClient>(window: &Window, ctx: &Ctx<C>) -> WindowTarget {
    WindowTarget {
        width: ctx.config.geometry.width,
        height: ctx.config.geometry.height,
    }
}

fn calculate_coordinates<C: NiriClient>(
    pos: SidebarPosition,
    dims: WindowTarget,
    screen: (i32, i32),
    stack_offset: i32,
    is_focused: bool,
    ctx: &Ctx<C>,
) -> (i32, i32) {
    let state = &ctx.state;
    let margins = &ctx.config.margins;
    let interaction = &ctx.config.interaction;
    let (sw, sh) = screen;
    let (w, h) = (dims.width, dims.height);

    let active_peek = if is_focused {
        interaction.get_focus_peek()
    } else {
        interaction.peek
    };

    match pos {
        SidebarPosition::Right => {
            let visible_x = sw - w - margins.right;
            let hidden_x = sw - active_peek;
            let x = if state.is_hidden { hidden_x } else { visible_x };

            let start_y = sh - h - margins.top;
            let y = start_y - stack_offset;
            (x, y)
        }
        SidebarPosition::Left => {
            let visible_x = margins.left;
            let hidden_x = -w + active_peek;
            let x = if state.is_hidden { hidden_x } else { visible_x };

            let start_y = sh - h - margins.top;
            let y = start_y - stack_offset;
            (x, y)
        }
        SidebarPosition::Bottom => {
            let start_x = margins.left;
            let x = start_x + stack_offset;

            let visible_y = sh - h - margins.bottom;
            let hidden_y = sh - active_peek;
            let y = if state.is_hidden { hidden_y } else { visible_y };
            (x, y)
        }
        SidebarPosition::Top => {
            let start_x = margins.left;
            let x = start_x + stack_offset;

            let visible_y = margins.top;
            let hidden_y = -h + active_peek;
            let y = if state.is_hidden { hidden_y } else { visible_y };
            (x, y)
        }
    }
}

pub fn reorder<C: NiriClient>(ctx: &mut Ctx<C>) -> Result<()> {
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

    let initial_len = ctx.state.windows.len();
    let active_ids: HashSet<u64> = all_windows.iter().map(|w| w.id).collect();
    ctx.state
        .windows
        .retain(|(id, _, _)| active_ids.contains(id));
    if ctx.state.windows.len() != initial_len {
        save_state(&ctx.state, &ctx.cache_dir)?;
    }

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

    let position = ctx.config.interaction.position;
    let gap = ctx.config.geometry.gap;

    let mut current_stack_offset = 0;

    for window in sidebar_windows.iter() {
        let dims = resolve_dimensions(window, ctx);

        let (target_x, target_y) = calculate_coordinates(
            position,
            dims,
            (display_w, display_h),
            current_stack_offset,
            window.is_focused,
            ctx,
        );

        match position {
            SidebarPosition::Left | SidebarPosition::Right => {
                current_stack_offset += dims.height + gap;
            }
            SidebarPosition::Top | SidebarPosition::Bottom => {
                current_stack_offset += dims.width + gap;
            }
        }

        let _ = ctx.socket.send_action(Action::MoveFloatingWindow {
            id: Some(window.id),
            x: PositionChange::SetFixed(target_x.into()),
            y: PositionChange::SetFixed(target_y.into()),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use crate::test_utils::{MockNiri, mock_config, mock_window};
    use niri_ipc::{Action, PositionChange};
    use tempfile::tempdir;

    #[test]
    fn test_standard_stacking_order() {
        let temp_dir = tempdir().unwrap();
        // Scenario: Two windows, visible. Check Y-axis stacking.
        let w1 = mock_window(1, false, true, 1);
        let w2 = mock_window(2, true, true, 1);
        let mock = MockNiri::new(vec![w1, w2]);

        let mut state = AppState::default();
        // 1 is bottom, 2 is top
        state.windows.push((1, 300, 200));
        state.windows.push((2, 300, 200));

        let mut ctx = Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        reorder(&mut ctx).expect("Reorder failed");

        let actions = &ctx.socket.sent_actions;
        assert_eq!(actions.len(), 2);

        // Screen W: 1920, H: 1080
        // Config: W: 300, H: 200, Gap: 10, Top: 50, Right: 20
        let base_x = 1920 - 300 - 20; // 1600
        let base_y = 1080 - 200 - 50; // 830 (Bottom-most slot)

        // Window 1 (Index 0)
        assert!(actions.iter().any(|a| matches!(a,
            Action::MoveFloatingWindow {
                id: Some(1),
                x: PositionChange::SetFixed(x),
                y: PositionChange::SetFixed(y)
            } if *x == f64::from(base_x) && *y == f64::from(base_y)
        )));

        // Window 2 (Index 1) -> Stacked above
        // Y = BaseY - (Height + Gap) = 830 - (200 + 10) = 620
        assert!(actions.iter().any(|a| matches!(a,
            Action::MoveFloatingWindow {
                id: Some(2),
                x: PositionChange::SetFixed(x),
                y: PositionChange::SetFixed(y)
            } if *x == f64::from(base_x) && *y == 620.0
        )));
    }

    #[test]
    fn test_hidden_mode_with_focus_peek() {
        let temp_dir = tempdir().unwrap();
        // Scenario: Hidden mode. Focused window should stick out more.
        let w_focused = mock_window(1, true, true, 1);
        let w_bg = mock_window(2, false, true, 1);
        let mock = MockNiri::new(vec![w_focused, w_bg]);

        let mut state = AppState {
            is_hidden: true,
            ..Default::default()
        };
        state.windows.push((1, 300, 200));
        state.windows.push((2, 300, 200));

        let mut ctx = Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        reorder(&mut ctx).expect("Reorder failed");

        let actions = &ctx.socket.sent_actions;

        // Config: Peek: 10, FocusPeek: 50
        // 1. Unfocused Window (ID 2) -> Should be at 1920 - 10 = 1910
        assert!(actions.iter().any(|a| matches!(a,
            Action::MoveFloatingWindow { id: Some(2), x: PositionChange::SetFixed(x), .. }
            if *x == 1910.0
        )));

        // 2. Focused Window (ID 1) -> Should be at 1920 - 50 = 1870
        assert!(actions.iter().any(|a| matches!(a,
            Action::MoveFloatingWindow { id: Some(1), x: PositionChange::SetFixed(x), .. }
            if *x == 1870.0
        )));
    }

    #[test]
    fn test_filters_wrong_workspace_and_cleanup_zombies() {
        let temp_dir = tempdir().unwrap();
        // Scenario:
        // - Window 1: On workspace 1 (Correct)
        // - Window 2: On workspace 99 (Should be ignored)
        // - Window 3: In State, but does not exist in Niri

        let w1 = mock_window(1, false, true, 1);
        let w2 = mock_window(2, false, true, 99);
        let mock = MockNiri::new(vec![w1, w2]);

        let mut state = AppState::default();
        state.windows.push((1, 100, 100));
        state.windows.push((2, 100, 100));
        state.windows.push((3, 100, 100));

        let mut ctx = Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        reorder(&mut ctx).unwrap();

        // Check Logic:
        // 1. Window 3 should be removed from state
        // 2. Window 2 should NOT be moved
        // 3. Window 1 SHOULD be moved

        let ids: Vec<u64> = ctx.state.windows.iter().map(|(id, _, _)| *id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(
            !ids.contains(&3),
            "Zombie window 3 should be removed from state"
        );

        // Assert Actions
        let actions = &ctx.socket.sent_actions;

        // Should move ID 1
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, Action::MoveFloatingWindow { id: Some(1), .. }))
        );
        // Should NOT move ID 2 (Wrong WS)
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, Action::MoveFloatingWindow { id: Some(2), .. }))
        );
        // Should NOT move ID 3 (Doesn't exist)
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, Action::MoveFloatingWindow { id: Some(3), .. }))
        );
    }

    #[test]
    fn test_flipped_order() {
        let temp_dir = tempdir().unwrap();
        // Scenario: Flipped mode reverses the visual stack
        let w1 = mock_window(1, false, true, 1);
        let w2 = mock_window(2, false, true, 1);
        let mock = MockNiri::new(vec![w1, w2]);

        let mut state = AppState {
            is_flipped: true,
            ..Default::default()
        };
        state.windows.push((1, 300, 200));
        state.windows.push((2, 300, 200));

        let mut ctx = Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        reorder(&mut ctx).unwrap();

        let actions = &ctx.socket.sent_actions;

        // Normal Order: 1 is bottom (idx 0), 2 is top (idx 1)
        // Flipped Order: 2 becomes bottom (idx 0), 1 becomes top (idx 1)
        // Check Window 2 is now at the Bottom (BaseY)
        // BaseY = 1080 - 200 - 50 = 830
        assert!(actions.iter().any(|a| matches!(a,
            Action::MoveFloatingWindow { id: Some(2), y: PositionChange::SetFixed(y), .. }
            if *y == 830.0
        )));
        // Check Window 1 is now stacked above
        // Y = 830 - (200 + 10) = 620
        assert!(actions.iter().any(|a| matches!(a,
            Action::MoveFloatingWindow { id: Some(1), y: PositionChange::SetFixed(y), .. }
            if *y == 620.0
        )));
    }
}
