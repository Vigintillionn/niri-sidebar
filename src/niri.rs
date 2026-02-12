use anyhow::{Context, Result, bail};
use niri_ipc::Action;
use niri_ipc::{Request, Response, socket::Socket};
pub use niri_ipc::{Window, Workspace};

pub trait NiriClient {
    fn get_windows(&mut self) -> Result<Vec<Window>>;
    fn get_active_window(&mut self) -> Result<Window>;
    fn get_active_workspace(&mut self) -> Result<Workspace>;
    fn get_screen_dimensions(&mut self) -> Result<(i32, i32)>;
    fn send_action(&mut self, action: Action) -> Result<Response>;
}

pub fn connect() -> Result<Socket> {
    Socket::connect().context("Failed to connect to Niri socket")
}

impl NiriClient for Socket {
    fn get_windows(&mut self) -> Result<Vec<Window>> {
        match self.send(Request::Windows)? {
            Ok(Response::Windows(windows)) => Ok(windows),
            _ => bail!("Unexpected response from Niri when fetching windows"),
        }
    }

    fn get_active_window(&mut self) -> Result<Window> {
        match self.send(Request::FocusedWindow)? {
            Ok(Response::FocusedWindow(Some(window))) => Ok(window),
            Ok(Response::FocusedWindow(None)) => bail!("No window focused"),
            _ => bail!("Unexpected response from Niri when fetching windows"),
        }
    }

    fn get_active_workspace(&mut self) -> Result<Workspace> {
        match self.send(Request::Workspaces)? {
            Ok(Response::Workspaces(workspaces)) => workspaces
                .into_iter()
                .find(|w| w.is_focused)
                .context("No active workspace found"),
            _ => bail!("Unexpected response from Niri when fetching workspaces"),
        }
    }

    fn get_screen_dimensions(&mut self) -> Result<(i32, i32)> {
        let workspace = self.get_active_workspace()?;
        let target_output_name = workspace
            .output
            .context("Focused workspace is not on an output")?;

        match self.send(Request::Outputs)? {
            Ok(Response::Outputs(outputs)) => {
                let output = outputs
                    .values()
                    .find(|o| o.name == target_output_name)
                    .context("Output not found")?;

                // Return the logical size
                let logical = output
                    .logical
                    .as_ref()
                    .context("Output has no logical size")?;
                Ok((
                    logical.width.try_into().unwrap_or(1920),
                    logical.height.try_into().unwrap_or(1080),
                ))
            }
            _ => bail!("Unexpected response from Niri when fetching outputs"),
        }
    }

    fn send_action(&mut self, action: Action) -> Result<Response> {
        self.send(Request::Action(action))?
            .map_err(|e| anyhow::anyhow!(e))
    }
}
