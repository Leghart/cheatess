use crate::utils::error::{CheatessError, CheatessResult};
use image::{ImageBuffer, Rgba};
pub use xcap::Monitor;

pub trait MonitorLike {
    fn name(&self) -> CheatessResult<String>;
    fn is_primary(&self) -> CheatessResult<bool>;
}

impl MonitorLike for Monitor {
    fn name(&self) -> CheatessResult<String> {
        Ok(self.name()?)
    }

    fn is_primary(&self) -> CheatessResult<bool> {
        Ok(self.is_primary()?)
    }
}

/// Selects a monitor by its name. If no name is provided, selects the primary monitor.
/// To see available monitor names, run `xrandr`.
pub fn select_monitor(name: Option<String>) -> CheatessResult<Monitor> {
    select_monitor_from_iterable(name, Monitor::all()?)
}

fn select_monitor_from_iterable<M: MonitorLike>(
    name: Option<String>,
    monitors: Vec<M>,
) -> CheatessResult<M> {
    match name {
        Some(n) => {
            if let Some(m) = monitors
                .into_iter()
                .find(|m| m.name().unwrap_or_default() == n)
            {
                log::info!("used monitor with name: {}", n);
                return Ok(m);
            }
        }
        None => {
            if let Some(m) = monitors
                .into_iter()
                .find(|m| m.is_primary().unwrap_or(false))
            {
                log::warn!("selected primary monitor by default");
                return Ok(m);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    struct MockMonitor {
        name: String,
        is_primary: bool,
    }

    impl MonitorLike for MockMonitor {
        fn name(&self) -> CheatessResult<String> {
            Ok(self.name.clone())
        }

        fn is_primary(&self) -> CheatessResult<bool> {
            Ok(self.is_primary)
        }
    }

    #[fixture]
    fn monitors() -> Vec<MockMonitor> {
        vec![
            MockMonitor {
                name: "Monitor1".to_string(),
                is_primary: false,
            },
            MockMonitor {
                name: "Monitor2".to_string(),
                is_primary: false,
            },
            MockMonitor {
                name: "Monitor3".to_string(),
                is_primary: true,
            },
        ]
    }

    #[rstest]
    fn test_select_monitor_by_name(monitors: Vec<MockMonitor>) {
        let selected_monitor =
            select_monitor_from_iterable(Some("Monitor2".to_string()), monitors).unwrap();
        assert_eq!(selected_monitor.name().unwrap(), "Monitor2");
    }

    #[rstest]
    fn test_select_primary_monitor(monitors: Vec<MockMonitor>) {
        let selected_monitor = select_monitor_from_iterable(None, monitors).unwrap();
        assert_eq!(selected_monitor.name().unwrap(), "Monitor3");
    }

    #[rstest]
    fn test_select_nonexisting_monitor(monitors: Vec<MockMonitor>) {
        let selected_monitor = select_monitor_from_iterable(Some("xxx".to_string()), monitors);

        assert!(selected_monitor.is_err());
        let err = selected_monitor.err().unwrap();
        assert!(matches!(err, CheatessError::MonitorNotFound));
    }
}
