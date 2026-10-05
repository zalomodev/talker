use crate::error::{AppError, AppResult};
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

pub fn set_click_through(window: &WebviewWindow, ignore: bool) -> AppResult<()> {
    window
        .set_ignore_cursor_events(ignore)
        .map_err(|e| AppError::Window(format!("Failed to set cursor event passthrough: {e}")))
}

pub fn set_bounds(
    window: &WebviewWindow,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> AppResult<()> {
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| AppError::Window(format!("Failed to set window position: {e}")))?;

    window
        .set_size(PhysicalSize::new(width, height))
        .map_err(|e| AppError::Window(format!("Failed to set window size: {e}")))?;

    Ok(())
}
