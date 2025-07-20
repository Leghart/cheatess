use image::{ImageBuffer, Rgba};
use xcap::Monitor;

#[allow(clippy::if_same_then_else)]
pub fn select_monitor(primary: bool) -> Result<Monitor, Box<dyn std::error::Error>> {
    for m in Monitor::all().unwrap() {
        if primary && m.is_primary().unwrap() {
            return Ok(m);
        } else if !primary && !m.is_primary().unwrap() {
            return Ok(m);
        }
    }
    Err("Monitor not found".into())
}

pub fn capture_entire_screen(monitor: &Monitor) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let capture = monitor.capture_image().unwrap();

    ImageBuffer::<Rgba<u8>, _>::from_raw(capture.width(), capture.height(), capture.into_vec())
        .unwrap()
}

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
