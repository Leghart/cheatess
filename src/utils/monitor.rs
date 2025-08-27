use crate::utils::error::{CheatessError, CheatessResult};
use image::{ImageBuffer, Rgba};
pub use xcap::Monitor;

#[allow(clippy::if_same_then_else)]
pub fn select_monitor(primary: u8) -> CheatessResult<Monitor> {
    for m in Monitor::all().unwrap() {
        if primary == 0 && m.is_primary().unwrap() {
            return Ok(m);
        } else if primary != 0 && !m.is_primary().unwrap() {
            //TODO: handle other monitors
            return Ok(m);
        }
    }

    Err(CheatessError::MonitorNotFound)
}

pub fn capture_entire_screen(monitor: &Monitor) -> CheatessResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let capture = monitor.capture_image()?;

    Ok(
        ImageBuffer::<Rgba<u8>, _>::from_raw(capture.width(), capture.height(), capture.into_vec())
            .expect("Failed to create ImageBuffer"),
    )
}

pub fn get_cropped_screen(
    monitor: &Monitor,
    x_start: u32,
    y_start: u32,
    width: u32,
    height: u32,
) -> CheatessResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let capture = monitor.capture_region(x_start, y_start, width, height)?;

    Ok(
        ImageBuffer::<Rgba<u8>, _>::from_raw(capture.width(), capture.height(), capture.into_vec())
            .expect("Failed to create ImageBuffer"),
    )
}
