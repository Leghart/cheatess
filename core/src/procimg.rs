use super::engine::{register_piece, Color};

use image::DynamicImage;
use opencv::{core::Mat, imgproc};
use opencv::{
    core::{min_max_loc, Point, Rect, Scalar, CV_8UC1, CV_8UC4},
    highgui::{self, destroy_window},
    prelude::*,
    Result,
};
use std::sync::{Arc, Mutex};
use std::thread;

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

#[allow(dead_code)]
pub fn show(image: &Mat, destroy: bool) -> Result<(), Box<dyn std::error::Error>> {
    highgui::imshow("test_window", &image)?;
    loop {
        if highgui::wait_key(0)? == 48 {
            break;
        }
    }
    if destroy {
        destroy_window("test_window")?;
    }
    Ok(())
}

/// Finds all chess pieces on the board by performing template matching for each piece.
/// It uses multithreading to speed up the process by processing each piece in a separate thread.
/// The function returns a 2D array representing the chessboard, where each cell contains the piece's symbol.
/// If a cell is empty, it contains a space character.
pub fn find_all_pieces(
    gray_board: &Mat,
    pieces: &std::collections::HashMap<char, Arc<Mat>>,
) -> [[char; 8]; 8] {
    let mut bin_board = Mat::default();
    imgproc::threshold(
        &gray_board,
        &mut bin_board,
        100.0, //TODO: downgraded from 127 to lichess
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
            let local_result = single_process(&board, &piece, 0.1, sign).unwrap();

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
    new_data
}

/// Processes a single chess piece image against a board image.
/// This function uses template matching to find the piece on the board and registers its position.
/// It updates the `result` array with the found piece's symbol at the corresponding board position
/// Both images (board and piece) are already binary thresholded, so mask is not needed.
fn single_process(
    board_image: &Mat,
    piece_image: &Mat,
    threshold: f64,
    symbol: char,
) -> Result<[[char; 8]; 8], Box<dyn std::error::Error>> {
    let mut result: [[char; 8]; 8] = [[' '; 8]; 8];
    let empty_mask = Mat::default();

    let mut matched = Mat::default();
    imgproc::match_template(
        board_image,
        piece_image,
        &mut matched,
        imgproc::TM_SQDIFF_NORMED,
        &empty_mask,
    )?;

    // TODO: for lichess
    // if symbol == 'P' {
    //     threshold = 0.05;
    // }

    let mut min_val = 0.0;
    let mut max_val = 0.0;
    let mut min_loc = Point::default();
    let mut max_loc = Point::default();

    let board_size = board_image.size()?;
    let matched_size = matched.size()?;
    let poison_val = Scalar::all(1.0);

    min_max_loc(
        &matched,
        Some(&mut min_val),
        Some(&mut max_val),
        Some(&mut min_loc),
        Some(&mut max_loc),
        &empty_mask,
    )?;

    loop {
        if min_val >= threshold {
            break;
        }

        let top_left = min_loc;

        register_piece(
            (top_left.y, top_left.x), // Note: OpenCV uses (y, x) for coordinates
            (board_size.width, board_size.height),
            symbol,
            &mut result,
        )?;

        let top_x = top_left.x.clamp(0, matched_size.width - 1);
        let top_y = top_left.y.clamp(0, matched_size.height - 1);

        let rect_x = (top_x - 22).max(0);
        let rect_y = (top_y - 22).max(0);
        let rect_w = 45.min(matched_size.width - rect_x);
        let rect_h = 45.min(matched_size.height - rect_y);

        let poison = Rect::new(rect_x, rect_y, rect_w, rect_h);

        let mut result_slice = matched.roi_mut(poison)?;
        result_slice.set_to(&poison_val, &empty_mask)?;

        min_max_loc(
            &matched,
            Some(&mut min_val),
            Some(&mut max_val),
            Some(&mut min_loc),
            Some(&mut max_loc),
            &empty_mask,
        )?;
    }

    Ok(result)
}

/// Detects the player's color by analyzing the bottom row of the chessboard.
/// It thresholds the grayscale image to create a binary image,
/// then checks the ratio of black pixels in the bottom row to determine if the player is playing with white or black pieces.
pub fn detect_player_color(gray_board: &Mat) -> Color {
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
        Color::Black
    } else {
        Color::White
    }
}

/// This function captures the screen and returns the region of the chessboard
/// with the following steps:
/// - Apply Canny edge detection to find the edges
/// - Find contours in the edge-detected image
/// - Approximate the contours to find quadrilaterals
pub fn get_board_region(gray: &Mat) -> (u32, u32, u32, u32) {
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
pub fn images_have_differences(gray1: &Mat, gray2: &Mat, threshold: i32) -> bool {
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
pub fn extract_pieces(
    img: &Mat,
    player_color: &Color,
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
        Color::White => &WHITE_NAMED_FIELDS,
        Color::Black => &BLACK_NAMED_FIELDS,
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
pub fn dynamic_image_to_gray_mat(img: &DynamicImage) -> opencv::Result<Mat> {
    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();

    let mut mat =
        Mat::new_rows_cols_with_default(height as i32, width as i32, CV_8UC4, Scalar::all(0.0))?;

    let mat_data = mat.data_bytes_mut()?;
    mat_data.copy_from_slice(&rgba8.as_raw());

    let mut gray_mat = Mat::default();
    imgproc::cvt_color(&mat, &mut gray_mat, imgproc::COLOR_RGBA2GRAY, 0)?;
    Ok(gray_mat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::imageops;

    #[test]
    fn get_board_region_from_entire_screenshot() {
        use image::{imageops, DynamicImage};
        use opencv::{imgcodecs, imgproc, prelude::*};

        let raw = imgcodecs::imread(
            "templates/boards/original/entire_board.png",
            imgcodecs::IMREAD_UNCHANGED,
        )
        .unwrap();

        let ref_mat = imgcodecs::imread(
            "templates/boards/original/gray_cropped.png",
            imgcodecs::IMREAD_UNCHANGED,
        )
        .unwrap();

        let mut gray_mat = Mat::default();
        imgproc::cvt_color(&raw, &mut gray_mat, imgproc::COLOR_RGBA2GRAY, 0).unwrap();

        let coords = get_board_region(&gray_mat);

        let rgba_image = {
            let size = raw.size().unwrap();
            image::RgbaImage::from_raw(
                size.width as u32,
                size.height as u32,
                raw.data_bytes().unwrap().to_vec(),
            )
            .unwrap()
        };

        let cropped =
            imageops::crop_imm(&rgba_image, coords.0, coords.1, coords.2, coords.3).to_image();

        let dyn_image = DynamicImage::ImageRgba8(cropped);
        let final_mat = dynamic_image_to_gray_mat(&dyn_image).unwrap();

        assert_eq!(final_mat.size().unwrap(), ref_mat.size().unwrap());
        assert_eq!(final_mat.typ(), ref_mat.typ());
    }
}
