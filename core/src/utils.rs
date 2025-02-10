use device_query::{DeviceQuery, DeviceState, MouseState};
use image::imageops::crop_imm;
use image::{ColorType, ImageBuffer, Luma, Rgba};
use screenshots::Screen;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Color {
    WHITE,
    BLACK,
}

#[derive(Debug, Default)]
pub struct ScreenArea {
    start_x: Option<i32>,
    start_y: Option<i32>,
    end_x: Option<i32>,
    end_y: Option<i32>,
}

fn detect_white_edges(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (u32, u32, u32, u32) {
    let (width, height) = image.dimensions();

    let mut x_min = width;
    let mut x_max = 0;
    let mut y_min = height;
    let mut y_max = 0;

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y)[0];

            if pixel > 220 {
                if x < x_min {
                    x_min = x;
                }
                if x > x_max {
                    x_max = x;
                }
                if y < y_min {
                    y_min = y;
                }
                if y > y_max {
                    y_max = y;
                }
            }
        }
    }

    (x_min, x_max, y_min, y_max)
}

pub fn trimm(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let (x_min, x_max, y_min, y_max) = detect_white_edges(image);

    let cropped = crop_imm(image, x_min, y_min, x_max - x_min, y_max - y_min).to_image();
    cropped
}

pub fn to_binary(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    threshold: u32,
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let (width, height) = image.dimensions();
    let mut binary_image = ImageBuffer::<Luma<u8>, Vec<u8>>::new(width, height);

    for (x, y, pixel) in image.enumerate_pixels() {
        let intensity = (pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3;
        let binary_value = if intensity > threshold { 255 } else { 0 };
        binary_image.put_pixel(x, y, Luma([binary_value]));
    }

    binary_image
}

pub fn get_screen_area() -> Result<ScreenArea, Box<dyn std::error::Error>> {
    let selection = Arc::new(Mutex::new(ScreenArea::default()));
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
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }
    let final_selection = Arc::try_unwrap(selection).unwrap().into_inner().unwrap();

    Ok(final_selection)
}

pub fn extract_board_state(
    image: ImageBuffer<Luma<u8>, Vec<u8>>,
) -> Result<[[Option<Color>; 8]; 8], &'static str> {
    let width = image.width();
    let height = image.height();
    let _vec = image.to_vec();

    let square_width = width / 8;
    let square_height = height / 8;

    let mut board: [[Option<Color>; 8]; 8] = [[None; 8]; 8];

    for row in 1..8 {
        let y = row * square_height;
        for x in 0..width {
            let pixel_index = (y * width + x) as usize;
            if _vec[pixel_index] == 0 {
                return Err("Detected a piece between squares on a horizontal grid line!");
            }
        }
    }

    for col in 1..8 {
        let x = col * square_width;
        for y in 0..height {
            let pixel_index = (y * width + x) as usize;
            if _vec[pixel_index] == 0 {
                return Err("Detected a piece between squares on a vertical grid line!");
            }
        }
    }

    for row in 0..8 {
        for col in 0..8 {
            let mut black_pixel_count = 0;
            let mut all = 0;

            for y in (row * square_height)..((row + 1) * square_height) {
                for x in (col * square_width)..((col + 1) * square_width) {
                    let pixel_index = (y * width + x) as usize; //TODO
                    if _vec[pixel_index] == 0 {
                        black_pixel_count += 1;
                    }
                    all += 1;
                }
            }

            // TODO: impl for white
            if (black_pixel_count * 100 / all) as f32 > 0.05 as f32 {
                // TODO!
                if black_pixel_count > 1800 {
                    board[row as usize][col as usize] = Some(Color::BLACK);
                } else {
                    board[row as usize][col as usize] = Some(Color::WHITE);
                }
            }
        }
    }

    Ok(board)
}

pub fn detect_move(
    before: &[[Option<Color>; 8]; 8],
    after: &[[Option<Color>; 8]; 8],
) -> ((usize, usize), (usize, usize)) {
    let mut start_pos = None;
    let mut end_pos = None;

    println!("before: {:?} after: {:?}", before, after);

    //TODO!: exf4 panic

    for row in 0..8 {
        for col in 0..8 {
            // moved to a new (empty) place
            if start_pos.is_none() && before[row][col].is_some() && after[row][col].is_none() {
                start_pos = Some((col, row)); // TODO: handle en passant
                continue;
            }

            // piece took another piece
            if end_pos.is_none() && before[row][col].is_some() && after[col][row].is_some() {
                if before[row][col] != after[row][col] {
                    println!("END 1 {col} {row}");
                    end_pos = Some((col, row));
                }
            } else if end_pos.is_none() && before[row][col].is_none() && after[row][col].is_some() {
                end_pos = Some((col, row));
            }

            if end_pos.is_some() && start_pos.is_some() {
                return (start_pos.unwrap(), end_pos.unwrap());
            }
        }
    }

    (
        start_pos.expect("Not found start piece position"),
        end_pos.expect("Not found end piece position"),
    )
}

pub fn pixel_to_chess_coord(x: usize, y: usize) -> String {
    let col = (b'a' + x as u8) as char; //  (a-h)
    let row = 8 - y; // (1-8)
    format!("{}{}", col, row)
}

pub fn check_if_board_was_changed(
    previous: &[[Option<Color>; 8]; 8],
    current: &[[Option<Color>; 8]; 8],
) -> bool {
    for row in 0..8 {
        for col in 0..8 {
            if previous[row][col] != current[row][col] {
                return true;
            }
        }
    }

    return false;
}

pub fn runner() {
    println!("Take a screenshot");
    let selection = get_screen_area().unwrap();
    let ref_image: ImageBuffer<Luma<u8>, Vec<u8>> = {
        let raw_image = take_screenshot(&selection).unwrap();
        let binary_image = to_binary(&raw_image, 70);
        trimm(&binary_image)
    };

    let mut previous_board = extract_board_state(ref_image).unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let start_time = std::time::Instant::now();

        let new_image = {
            let raw_image = take_screenshot(&selection).unwrap();
            let bin = to_binary(&raw_image, 70);
            trimm(&bin)
            // TODO: change trim to updated selecion area with trimmed dimensions
        };
        new_image.save(std::path::Path::new("updated.png")).unwrap(); // tmp

        let current_result = extract_board_state(new_image); //TODO: missing info about piece color

        if current_result.is_err() {
            continue;
        }

        let current = current_result.unwrap();

        if check_if_board_was_changed(&previous_board, &current) {
            let (start, end) = detect_move(&previous_board, &current);
            let start_pos = pixel_to_chess_coord(start.0, start.1);
            let end_pos = pixel_to_chess_coord(end.0, end.1);
            println!("{} -> {} [{:?}]", start_pos, end_pos, start_time.elapsed());
            previous_board = current;
        }
    }
    println!("STOP");
}

pub fn take_screenshot(
    selection: &ScreenArea,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
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

        let img = screen
            .capture_area(x, y, width as u32, height as u32)
            .unwrap();

        return Ok(img);
    }
    panic!()
}

pub fn test_save_img(ndata: &[u8], width: u32, height: u32) {
    image::save_buffer(
        &Path::new("test_img.png"),
        ndata,
        width,
        height,
        ColorType::L8,
    )
    .unwrap();
}
