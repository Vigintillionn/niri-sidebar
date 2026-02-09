use anyhow::{Context, Result, bail};
use niri_ipc::{Request, Response, socket::Socket};
pub use niri_ipc::{Window, Workspace};

pub fn connect() -> Result<Socket> {
    Socket::connect().context("Failed to connect to Niri socket")
}

pub fn get_windows(socket: &mut Socket) -> Result<Vec<Window>> {
    match socket.send(Request::Windows)? {
        Ok(Response::Windows(windows)) => Ok(windows),
        _ => bail!("Unexpected response from Niri when fetching windows"),
    }
}

pub fn get_active_window(socket: &mut Socket) -> Result<Window> {
    let windows = get_windows(socket)?;
    windows
        .into_iter()
        .find(|w| w.is_focused)
        .context("No focused window.")
}

pub fn get_active_workspace(socket: &mut Socket) -> Result<Workspace> {
    match socket.send(Request::Workspaces)? {
        Ok(Response::Workspaces(workspaces)) => workspaces
            .into_iter()
            .find(|w| w.is_focused)
            .context("No active workspace found"),
        _ => bail!("Unexpected response from Niri when fetching workspaces"),
    }
}

pub fn get_active_workspace_id(socket: &mut Socket) -> Result<u64> {
    let current_ws = get_active_workspace(socket)?;
    Ok(current_ws.id)
}

pub fn get_screen_dimensions(socket: &mut Socket) -> Result<(i32, i32)> {
    let workspace = get_active_workspace(socket)?;
    let target_output_name = workspace
        .output
        .context("Focused workspace is not on an output")?;

    match socket.send(Request::Outputs)? {
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
