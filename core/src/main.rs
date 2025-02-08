use device_query::{DeviceQuery, DeviceState, MouseState};
use screenshots::Screen;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

#[derive(Debug, Default)]
struct Selection {
    start_x: Option<i32>,
    start_y: Option<i32>,
    end_x: Option<i32>,
    end_y: Option<i32>,
}

fn screenshot() {
    let selection = Arc::new(Mutex::new(Selection::default()));
    let device_state = DeviceState::new();
    let mut selecting = false;

    loop {
        let mouse: MouseState = device_state.get_mouse();
        let (x, y) = (mouse.coords.0 as i32, mouse.coords.1 as i32);

        // TODO: tmp bind with int
        if mouse.button_pressed[1] {
            if !selecting {
                let mut sel = selection.lock().unwrap();
                sel.start_x = Some(x);
                sel.start_y = Some(y);
                selecting = true;
            }
        } else if selecting {
            let mut sel = selection.lock().unwrap();
            sel.end_x = Some(x);
            sel.end_y = Some(y);
            selecting = false;
            if capture_selection(&sel).is_ok() {
                break;
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn main() {
    screenshot();
}

fn capture_selection(selection: &Selection) -> Result<(), Box<dyn std::error::Error>> {
    if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
        selection.start_x,
        selection.start_y,
        selection.end_x,
        selection.end_y,
    ) {
        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x2 - x1).abs();
        let height = (y2 - y1).abs();

        let screens = Screen::all().unwrap();
        let screen = screens.get(0).unwrap(); //TODO: works only for primary screen
        println!("for img: {x} {y} {} {}", width as u32, height as u32);
        let image = screen
            .capture_area(x, y, width as u32, height as u32)
            .unwrap();

        image.save("dupa.png").unwrap();
        return Ok(());
    }
    Err("dupa".into())
}
