use image::{ImageBuffer, Rgba};
use xcap::Monitor;

/// Selects a monitor based on whether it is primary or not.
/// If `primary` is true, it returns the primary monitor.
/// If `primary` is false, it returns the first non-primary monitor found.
#[allow(clippy::if_same_then_else)]
pub fn select_monitor(primary: bool) -> Option<Monitor> {
    for m in Monitor::all().unwrap() {
        if primary && m.is_primary().unwrap() {
            return Some(m);
        } else if !primary && !m.is_primary().unwrap() {
            return Some(m);
        }
    }
    None
}

/// Captures the entire screen of the specified monitor and returns it as an ImageBuffer.
pub fn capture_entire_screen(monitor: &Monitor) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let capture = monitor.capture_image().unwrap();

    ImageBuffer::<Rgba<u8>, _>::from_raw(capture.width(), capture.height(), capture.into_vec())
        .unwrap()
}

/// Captures a specific region of the screen defined by the starting coordinates (x_start, y_start)
/// and the dimensions (width, height).
pub fn get_cropped_screen(
    monitor: &Monitor,
    x_start: u32,
    y_start: u32,
    width: u32,
    height: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let capture = monitor
        .capture_region(x_start, y_start, width, height)
        .unwrap();

    ImageBuffer::<Rgba<u8>, _>::from_raw(capture.width(), capture.height(), capture.into_vec())
        .unwrap()
}
