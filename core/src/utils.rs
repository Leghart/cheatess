use device_query::{DeviceQuery, DeviceState, MouseState};
use image::imageops::crop_imm;
use image::{imageops, ColorType, DynamicImage, ImageBuffer, Luma, Rgba};
use opencv::core::Vector;
use opencv::{
    core::{Mat, Point, Scalar, Size, CV_8UC1},
    imgcodecs, imgproc,
    prelude::*,
    types,
};

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

pub fn detect_contours(image: &Mat) -> opencv::Result<Vector<Vector<Point>>> {
    let mut contours = Vector::new();
    imgproc::find_contours(
        &image,
        &mut contours,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        Point::new(0, 0),
    )?;

    Ok(contours)
}

pub fn compare_contours(contours1: &Vector<Vector<Point>>, contours2: &Vector<Vector<Point>>) {
    let mut moved_figures = vec![];

    for contour1 in contours1.iter() {
        let mut match_found = false;
        for contour2 in contours2.iter() {
            let similarity =
                imgproc::match_shapes(&contour1, &contour2, imgproc::CONTOURS_MATCH_I1, 0.0)
                    .unwrap();

            if similarity < 0.1 {
                match_found = true;
                break;
            }
        }

        if !match_found {
            moved_figures.push(contour1);
        }
    }

    if !moved_figures.is_empty() {
        println!("Znaleziono figury, które się poruszyły!");
    } else {
        println!("Brak ruchu figury.");
    }
}

pub fn save_mat(image: &Mat, path: &str) {
    let params = Vector::from_iter([16, 0]);
    imgcodecs::imwrite(path, &image, &params);
}

pub fn resize(image: &Mat, new_width: i32, new_height: i32) -> opencv::Result<Mat> {
    let new_size = Size::new(new_width, new_height);

    let mut resized_img = Mat::default();
    imgproc::resize(
        &image,
        &mut resized_img,
        new_size,
        0.0,
        0.0,
        imgproc::INTER_LINEAR,
    )?;

    Ok(resized_img)
}

pub fn to_binary(image: &Mat) -> Mat {
    let mut gray_image = Mat::default();
    imgproc::cvt_color(image, &mut gray_image, imgproc::COLOR_BGR2GRAY, 0).unwrap();

    let mut binary_image = Mat::default();
    imgproc::threshold(
        &gray_image,
        &mut binary_image,
        127.0,
        255.0,
        imgproc::THRESH_BINARY,
    )
    .unwrap();
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
    let _vec = image.clone().to_vec(); //tmp

    let square_width = width / 8;
    let square_height = height / 8;

    let mut board: [[Option<Color>; 8]; 8] = [[None; 8]; 8];

    let mut black_pixel_counts: [[u32; 8]; 8] = [[0; 8]; 8];
    let mut black_pixels: Vec<u32> = Vec::new();
    let mut white_pixels: Vec<u32> = Vec::new();
    let mut populate_black = true;

    for row in 0..8 {
        for col in 0..8 {
            let mut black_pixel_count: u32 = 0;

            // count pixels in each board square
            for y in (row * square_height)..((row + 1) * square_height) {
                for x in (col * square_width)..((col + 1) * square_width) {
                    let pixel_index = (y * width + x) as usize; //TODO
                    if _vec[pixel_index] == 0 {
                        black_pixel_count += 1;
                    }
                }
            }
            black_pixel_counts[row as usize][col as usize] = black_pixel_count;
            if black_pixel_count == 0 {
                populate_black = false;
            }

            if populate_black {
                black_pixels.push(black_pixel_count);
            } else {
                white_pixels.push(black_pixel_count);
            }
        }
    }

    let min_pivot = black_pixels.iter().min().unwrap();
    let max_pivot = white_pixels.iter().max().unwrap();

    for row in 0..8 {
        for col in 0..8 {
            board[row][col] = {
                if black_pixel_counts[row][col] >= *min_pivot {
                    Some(Color::BLACK)
                } else if black_pixel_counts[row][col] >= *max_pivot {
                    Some(Color::WHITE)
                } else {
                    None
                }
            }
        }
    }

    Ok(board)
}

pub fn detect_move(
    before: &[[Option<Color>; 8]; 8],
    after: &[[Option<Color>; 8]; 8],
) -> Result<((usize, usize), (usize, usize)), &'static str> {
    let mut start_pos = None;
    let mut end_pos = None;

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
                    end_pos = Some((col, row));
                }
            } else if end_pos.is_none() && before[row][col].is_none() && after[row][col].is_some() {
                end_pos = Some((col, row));
            }

            if end_pos.is_some() && start_pos.is_some() {
                return Ok((start_pos.unwrap(), end_pos.unwrap()));
            }
        }
    }

    if start_pos.is_some() && end_pos.is_some() {
        return Ok((start_pos.unwrap(), end_pos.unwrap()));
    }

    Err("Could not match pieces to move")
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

// pub fn runner() {
//     println!("Take a screenshot");
//     let selection = get_screen_area().unwrap();
//     println!("Time to play the gaaaaame...");
//     let ref_image: ImageBuffer<Luma<u8>, Vec<u8>> = {
//         let raw_image = take_screenshot(&selection).unwrap();
//         let binary_image = to_binary(&raw_image, 30);
//         trimm(&binary_image)
//         // binary_image
//     };

//     let mut previous_board = extract_board_state(ref_image).unwrap();

//     loop {
//         std::thread::sleep(std::time::Duration::from_millis(100));
//         let start_time = std::time::Instant::now();

//         let new_image = {
//             let _raw_image = take_screenshot(&selection).unwrap();
//             let _binary_image = to_binary(&_raw_image, 30);
//             trimm(&_binary_image)
//             // _binary_image
//             // TODO: change trim to updated selecion area with trimmed dimensions
//         };
//         new_image.save(std::path::Path::new("updated.png")).unwrap(); // tmp

//         let current_result = extract_board_state(new_image);

//         if current_result.is_err() {
//             let e = current_result.err().unwrap();
//             println!("Border error {:?}", e);
//             continue;
//         }

//         let current = current_result.unwrap();

//         if check_if_board_was_changed(&previous_board, &current) {
//             let move_result = detect_move(&previous_board, &current);
//             if move_result.is_err() {
//                 // previous_board = current;
//                 let e = move_result.err().unwrap();
//                 println!("Move error: {e}");
//                 println!("BEFORE: {:?}", previous_board);
//                 println!("AFTER: {:?}", current);
//                 // break;
//                 continue;
//             }
//             let (start, end) = move_result.unwrap();
//             let start_pos = pixel_to_chess_coord(start.0, start.1);
//             let end_pos = pixel_to_chess_coord(end.0, end.1);
//             println!("{} -> {} [{:?}]", start_pos, end_pos, start_time.elapsed());
//             previous_board = current;
//         }
//     }
// }

pub fn to_mat(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Mat {
    let (img_width, img_height) = image.dimensions();
    let img_data = image.to_vec();

    let mat = Mat::new_rows_cols_with_data(img_height as i32, img_width as i32, &img_data).unwrap();
    mat.try_clone().unwrap()
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
