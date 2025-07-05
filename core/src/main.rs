use image::{imageops, DynamicImage, ImageBuffer, Rgba};
use opencv::{
    core::{Mat, Point, Rect, Scalar, CV_8UC1, CV_8UC4},
    imgproc,
    prelude::*,
    Result,
};
use std::io::stdout;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use xcap::Monitor;
mod engine;
mod img_proc;
mod stockfish;

static WHITE_NAMED_FIELDS: [((usize, usize), char); 12] = [
    ((0, 0), 'r'),
    ((1, 0), 'n'),
    ((2, 0), 'b'),
    ((3, 0), 'q'),
    ((4, 0), 'k'),
    ((0, 1), 'p'),
    ((0, 6), 'P'),
    ((0, 7), 'R'),
    ((1, 7), 'N'),
    ((2, 7), 'B'),
    ((3, 7), 'Q'),
    ((4, 7), 'K'),
];

static BLACK_NAMED_FIELDS: [((usize, usize), char); 12] = [
    ((0, 0), 'R'),
    ((1, 0), 'N'),
    ((2, 0), 'B'),
    ((3, 0), 'K'),
    ((4, 0), 'Q'),
    ((0, 1), 'P'),
    ((0, 6), 'p'),
    ((0, 7), 'r'),
    ((1, 7), 'n'),
    ((2, 7), 'b'),
    ((3, 7), 'k'),
    ((4, 7), 'q'),
];

fn main() {
    run();
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}

fn run() {
    let mut st =
        stockfish::Stockfish::new("/home/leghart/projects/cheatess/stockfish-ubuntu-x86-64-avx2");
    st.set_elo_rating(2800);

    let monitor = select_monitor(true).expect("No primary monitor found");
    let raw = capture_entire_screen(&monitor);
    let dyn_image = DynamicImage::ImageRgba8(raw.clone());
    let entire_screen_gray = dynamic_image_to_gray_mat(&dyn_image).unwrap();

    let coords = get_board_region(&entire_screen_gray);

    let cropped = imageops::crop_imm(&raw, coords.0, coords.1, coords.2, coords.3).to_image();
    let dyn_image = DynamicImage::ImageRgba8(cropped.clone());
    let board = dynamic_image_to_gray_mat(&dyn_image).unwrap();

    let player_color = detect_player_color(&board);

    let base_board = engine::create_board::<engine::PrettyPrinter>(&player_color);
    base_board.print();

    let pieces = extract_pieces(&board, player_color).unwrap();

    let pieces: Arc<Vec<(char, Arc<Mat>)>> = Arc::new(
        pieces
            .into_iter()
            .map(|(sign, piece)| (sign, Arc::new(piece)))
            .collect(),
    );

    let mut prev_board_mat = board;
    let mut prev_board_arr = base_board;
    loop {
        let start = Instant::now();
        let cropped = get_cropped_screen(&monitor, coords.0, coords.1, coords.2, coords.3);
        let dyn_image = DynamicImage::ImageRgba8(cropped);
        let gray_board = dynamic_image_to_gray_mat(&dyn_image).unwrap();

        if !images_have_differences(&prev_board_mat, &gray_board, 500) {
            continue;
        }

        let mut bin_board = Mat::default();
        imgproc::threshold(
            &gray_board,
            &mut bin_board,
            127.0,
            255.0,
            imgproc::THRESH_BINARY,
        )
        .unwrap();

        let result = Arc::new(Mutex::new([[' '; 8]; 8]));
        let bin_board = Arc::new(bin_board);

        let mut handles = vec![];
        for (sign, piece_arc) in pieces.iter() {
            let board = Arc::clone(&bin_board);
            let result_ref = Arc::clone(&result);
            let piece = Arc::clone(piece_arc);
            let sign = *sign;

            let handle = thread::spawn(move || {
                let local_result = img_proc::single_process(&board, &piece, 0.1, sign).unwrap();

                let mut res = result_ref.lock().unwrap();
                for row in 0..8 {
                    for col in 0..8 {
                        if local_result[row][col] != ' ' && res[row][col] == ' ' {
                            res[row][col] = local_result[row][col];
                        }
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
        let new_data = *result.lock().unwrap();
        let detected_move = engine::detect_move(&prev_board_arr.board, &new_data);

        if let Some(mv) = detected_move {
            println!("Detected move: {:?}", mv);
            st.make_move(vec![mv]);
        } else {
            println!("not found move");
        }

        clear_screen();
        let curr_board = engine::Board::new(new_data);
        curr_board.print();
        let best_move = st.get_best_move().unwrap();
        println!("Stockfish best move: {}", best_move);
        stdout().flush().unwrap();

        prev_board_arr = curr_board;
        prev_board_mat = gray_board;
        println!("Time taken: {:?}", start.elapsed());
    }
}

/// Detects the player's color by analyzing the bottom row of the chessboard.
/// It thresholds the grayscale image to create a binary image,
/// then checks the ratio of black pixels in the bottom row to determine if the player is playing with white or black pieces.
fn detect_player_color(gray_board: &Mat) -> engine::Color {
    let mut bin_board = Mat::default();
    imgproc::threshold(
        &gray_board,
        &mut bin_board,
        50.0,
        255.0,
        imgproc::THRESH_BINARY,
    )
    .unwrap();

    let img = bin_board;
    let width = img.cols();
    let height = img.rows();
    let square_width = width / 8;
    let square_height = height / 8;

    let roi = Rect::new(0, height - square_height, square_width, square_height);
    let square = Mat::roi(&img, roi).unwrap();

    let mut square_continuous = Mat::new_rows_cols_with_default(
        square.rows(),
        square.cols(),
        CV_8UC1,
        opencv::core::Scalar::all(0.0),
    )
    .unwrap();

    square.copy_to(&mut square_continuous).unwrap();

    let total_pixels = square_continuous.rows() * square_continuous.cols();
    let black_pixels = square_continuous
        .data_bytes()
        .unwrap()
        .iter()
        .filter(|&&p| p == 0)
        .count();

    let black_ratio = black_pixels as f32 / total_pixels as f32;

    // white rook: 0.14, black rook: 0.26
    if black_ratio > 0.2 {
        engine::Color::Black
    } else {
        engine::Color::White
    }
}

/// Selects a monitor based on whether it is primary or not.
/// If `primary` is true, it returns the primary monitor.
/// If `primary` is false, it returns the first non-primary monitor found.
fn select_monitor(primary: bool) -> Option<Monitor> {
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
fn capture_entire_screen(monitor: &Monitor) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let capture = monitor.capture_image().unwrap();

    ImageBuffer::<Rgba<u8>, _>::from_raw(capture.width(), capture.height(), capture.into_vec())
        .unwrap()
}

/// Captures a specific region of the screen defined by the starting coordinates (x_start, y_start)
/// and the dimensions (width, height).
fn get_cropped_screen(
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

/// This function captures the screen and returns the region of the chessboard
/// with the following steps:
/// - Apply Canny edge detection to find the edges
/// - Find contours in the edge-detected image
/// - Approximate the contours to find quadrilaterals
fn get_board_region(gray: &Mat) -> (u32, u32, u32, u32) {
    // contours without blurring to keep sharp edges
    let mut edges = Mat::default();
    imgproc::canny(&gray, &mut edges, 50.0, 150.0, 3, false).unwrap();

    let mut contours = opencv::core::Vector::<opencv::core::Vector<Point>>::new();
    imgproc::find_contours(
        &edges,
        &mut contours,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        Point::new(0, 0),
    )
    .unwrap();

    let mut max_area = 0.0;
    let mut best_quad = vec![];

    for contour in contours {
        let mut approx = opencv::core::Vector::<Point>::new();
        imgproc::approx_poly_dp(
            &contour,
            &mut approx,
            0.02 * imgproc::arc_length(&contour, true).unwrap(),
            true,
        )
        .unwrap();

        if approx.len() == 4 && imgproc::is_contour_convex(&approx).unwrap() {
            let area = imgproc::contour_area(&approx, false).unwrap();
            let bounding = imgproc::bounding_rect(&approx).unwrap();

            let aspect_ratio = bounding.width as f32 / bounding.height as f32;
            if area > max_area && aspect_ratio > 0.8 && aspect_ratio < 1.2 {
                max_area = area;
                best_quad = approx.to_vec();
            }
        }
    }

    let x_start = best_quad[0].x as u32;
    let y_start = best_quad[0].y as u32;
    let width = best_quad[2].x as u32 - x_start;
    let height = best_quad[2].y as u32 - y_start;

    (x_start, y_start, width, height)
}

/// Checks if two images have differences in their 8x8 grid cells.
/// It divides the images into 8x8 cells and checks if the number of non-zero
/// pixels in each cell exceeds a given threshold.
fn images_have_differences(gray1: &Mat, gray2: &Mat, threshold: i32) -> bool {
    let cell_w = gray1.cols() / 8;
    let cell_h = gray1.rows() / 8;

    for row in 0..8 {
        for col in 0..8 {
            let x = col * cell_w;
            let y = row * cell_h;

            let width = if col == 7 { gray1.cols() - x } else { cell_w };
            let height = if row == 7 { gray1.rows() - y } else { cell_h };

            let roi = Rect::new(x, y, width, height);
            let patch1 = Mat::roi(gray1, roi).unwrap();
            let patch2 = Mat::roi(gray2, roi).unwrap();

            let mut thresh1 = Mat::default();
            imgproc::threshold(
                &patch1,
                &mut thresh1,
                50.0,
                255.0,
                imgproc::THRESH_BINARY_INV,
            )
            .unwrap();

            let mut thresh2 = Mat::default();
            imgproc::threshold(
                &patch2,
                &mut thresh2,
                50.0,
                255.0,
                imgproc::THRESH_BINARY_INV,
            )
            .unwrap();

            let nonzero1 = opencv::core::count_non_zero(&thresh1).unwrap();
            let nonzero2 = opencv::core::count_non_zero(&thresh2).unwrap();

            if (nonzero1 > threshold) != (nonzero2 > threshold) {
                return true;
            }
        }
    }

    false
}

/// Extracts pieces from the chessboard image.
/// It divides the board into 8x8 cells and extracts the pieces based on predefined positions.
/// It applies a margin to the extraction area to avoid cutting off pieces.
fn extract_pieces(
    img: &Mat,
    player_color: engine::Color,
) -> Result<std::collections::HashMap<char, Mat>, Box<dyn std::error::Error>> {
    let board_size: i32 = img.rows().min(img.cols());
    let board_size_f = board_size as f32;

    let mut x_edges = [0i32; 9];
    let mut y_edges = [0i32; 9];

    for i in 0..=8 {
        x_edges[i] = ((i as f32) * board_size_f / 8.0).round() as i32;
        y_edges[i] = ((i as f32) * board_size_f / 8.0).round() as i32;
    }

    let named_fields = match player_color {
        engine::Color::White => &WHITE_NAMED_FIELDS,
        engine::Color::Black => &BLACK_NAMED_FIELDS,
    };

    let mut result = std::collections::HashMap::new();
    for ((col, row), name) in named_fields {
        let x = x_edges[*col];
        let y = y_edges[*row];
        let w = x_edges[col + 1] - x;
        let h = y_edges[row + 1] - y;

        // Add a margin to the piece extraction area
        let margin = 5;
        let x = x + margin;
        let y = y + margin;
        let w = (w - 2 * margin).max(1);
        let h = (h - 2 * margin).max(1);

        let roi = Rect::new(x, y, w, h);
        let piece = Mat::roi(img, roi)?;

        let mut bin_piece = Mat::default();
        imgproc::threshold(&piece, &mut bin_piece, 127.0, 255.0, imgproc::THRESH_BINARY)?;
        result.insert(*name, bin_piece);
    }
    Ok(result)
}

/// Converts a DynamicImage to a grayscale Mat.
/// It first converts the image to RGBA8 format, then creates a Mat from the pixel data.
/// Finally, it converts the RGBA Mat to a grayscale Mat using OpenCV's cvt_color function.
fn dynamic_image_to_gray_mat(img: &DynamicImage) -> opencv::Result<Mat> {
    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();

    let mut mat =
        Mat::new_rows_cols_with_default(height as i32, width as i32, CV_8UC4, Scalar::all(0.0))
            .unwrap();

    let mat_data = mat.data_bytes_mut().unwrap();
    mat_data.copy_from_slice(&rgba8.as_raw());

    let mut gray_mat = Mat::default();
    imgproc::cvt_color(&mat, &mut gray_mat, imgproc::COLOR_RGBA2GRAY, 0).unwrap();
    Ok(gray_mat)
}
