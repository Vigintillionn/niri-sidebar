use crate::Ctx;
use crate::commands::reorder;
use crate::niri::NiriClient;
use crate::state::save_state;
use crate::Direction;
use anyhow::Result;

pub fn cycle<C: NiriClient>(ctx: &mut Ctx<C>, direction: Direction) -> Result<()> {
    if ctx.state.windows.len() < 2 {
        return Ok(());
    }

    match direction {
        Direction::Next => {
            let first = ctx.state.windows.remove(0);
            ctx.state.windows.push(first);
        }
        Direction::Prev => {
            let last = ctx.state.windows.pop().unwrap();
            ctx.state.windows.insert(0, last);
        }
    }

    save_state(&ctx.state, &ctx.cache_dir)?;
    reorder(ctx)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, WindowState};
    use crate::test_utils::{MockNiri, mock_config, mock_window};
    use tempfile::tempdir;

    fn make_ctx(temp_dir: &tempfile::TempDir) -> Ctx<MockNiri> {
        let mut state = AppState::default();
        for id in [1, 2, 3] {
            state.windows.push(WindowState {
                id,
                width: 300,
                height: 200,
                is_floating: true,
                position: Some((1.0, 2.0)),
            });
        }

        let mock = MockNiri::new(vec![
            mock_window(1, false, true, 1, Some((1.0, 2.0))),
            mock_window(2, false, true, 1, Some((1.0, 2.0))),
            mock_window(3, false, true, 1, Some((1.0, 2.0))),
        ]);

        Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        }
    }

    #[test]
    fn test_cycle_next() {
        let temp_dir = tempdir().unwrap();
        let mut ctx = make_ctx(&temp_dir);

        // [1, 2, 3] -> [2, 3, 1]
        cycle(&mut ctx, Direction::Next).unwrap();
        let ids: Vec<u64> = ctx.state.windows.iter().map(|w| w.id).collect();
        assert_eq!(ids, vec![2, 3, 1]);

        // Reorder should have been called
        assert!(!ctx.socket.sent_actions.is_empty());
    }

    #[test]
    fn test_cycle_prev() {
        let temp_dir = tempdir().unwrap();
        let mut ctx = make_ctx(&temp_dir);

        // [1, 2, 3] -> [3, 1, 2]
        cycle(&mut ctx, Direction::Prev).unwrap();
        let ids: Vec<u64> = ctx.state.windows.iter().map(|w| w.id).collect();
        assert_eq!(ids, vec![3, 1, 2]);

        assert!(!ctx.socket.sent_actions.is_empty());
    }

    #[test]
    fn test_cycle_single_window_noop() {
        let temp_dir = tempdir().unwrap();
        let mut state = AppState::default();
        state.windows.push(WindowState {
            id: 1,
            width: 300,
            height: 200,
            is_floating: true,
            position: Some((1.0, 2.0)),
        });

        let mock = MockNiri::new(vec![mock_window(1, false, true, 1, Some((1.0, 2.0)))]);

        let mut ctx = Ctx {
            state,
            config: mock_config(),
            socket: mock,
            cache_dir: temp_dir.path().to_path_buf(),
        };

        cycle(&mut ctx, Direction::Next).unwrap();
        assert_eq!(ctx.state.windows.len(), 1);
        assert_eq!(ctx.state.windows[0].id, 1);
        assert!(ctx.socket.sent_actions.is_empty());
    }
}
