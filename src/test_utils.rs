use crate::config::{Geometry, Margins};
use crate::{Config, NiriClient};
use anyhow::Result;
use niri_ipc::{Action, Response, Window, WindowLayout, Workspace};

#[derive(Default, Debug, Clone)]
pub struct MockNiri {
    pub windows: Vec<Window>,
    pub sent_actions: Vec<Action>,
}

impl MockNiri {
    pub fn new(windows: Vec<Window>) -> Self {
        Self {
            windows,
            sent_actions: vec![],
        }
    }
}

impl NiriClient for MockNiri {
    fn get_windows(&mut self) -> Result<Vec<Window>> {
        Ok(self.windows.clone())
    }

    fn get_active_window(&mut self) -> Result<Window> {
        self.windows
            .iter()
            .find(|w| w.is_focused)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No active window in mock"))
    }

    fn send_action(&mut self, action: Action) -> Result<Response> {
        self.sent_actions.push(action);
        Ok(Response::Handled)
    }

    fn get_active_workspace(&mut self) -> Result<Workspace> {
        Ok(Workspace {
            id: 1,
            idx: 0,
            name: Some("test".into()),
            output: Some("eDP-1".into()),
            is_urgent: false,
            is_active: true,
            is_focused: true,
            active_window_id: None,
        })
    }

    fn get_screen_dimensions(&mut self) -> Result<(i32, i32)> {
        Ok((1920, 1080))
    }
}

pub fn mock_window(
    id: u64,
    is_focused: bool,
    is_floating: bool,
    workspace_id: u64,
    position: Option<(f64, f64)>,
) -> Window {
    Window {
        id,
        is_focused,
        is_floating,
        workspace_id: Some(workspace_id),
        title: Some("Test Window".into()),
        app_id: Some("test".into()),
        pid: Some(123),
        is_urgent: false,
        layout: WindowLayout {
            window_size: (1000, 800),
            pos_in_scrolling_layout: None,
            tile_size: (0.0, 0.0),
            tile_pos_in_workspace_view: position,
            window_offset_in_tile: (0.0, 0.0),
        },
        focus_timestamp: None,
    }
}

pub fn mock_config() -> Config {
    Config {
        geometry: Geometry {
            width: 300,
            height: 200,
            gap: 10,
        },
        margins: Margins {
            top: 50,
            right: 20,
            left: 10,
            bottom: 50,
        },
        ..Default::default()
    }
}
