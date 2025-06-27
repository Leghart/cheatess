mod config;
mod engine;
mod img_proc;
mod stockfish;
mod utils;

use std::time::Instant;

use opencv::{
    core::{Mat, Point, Rect, Scalar, CV_8UC4},
    imgproc,
    prelude::*,
};
use std::sync::{Arc, Mutex};
use std::thread;

use image::{imageops, DynamicImage};
use xcap::Monitor;

use image::{ImageBuffer, Rgba};

use opencv::Result;

fn select_primary_monitor(primary: bool) -> Option<Monitor> {
    for m in Monitor::all().unwrap() {
        if primary && m.is_primary().unwrap() {
            return Some(m);
        } else if !primary && !m.is_primary().unwrap() {
            return Some(m);
        }
    }
    None
}

fn capture_entire_screen(monitor: &Monitor) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let capture = monitor.capture_image().unwrap();

    ImageBuffer::<Rgba<u8>, _>::from_raw(capture.width(), capture.height(), capture.into_vec())
        .unwrap()
}

// This function captures the screen and returns the region of the chessboard
// with the following steps:
// - Convert the captured image to grayscale
// - Apply Canny edge detection to find the edges
// - Find contours in the edge-detected image
// - Approximate the contours to find quadrilaterals
fn get_board_region(raw: &Mat) -> (u32, u32, u32, u32) {
    let mut gray = Mat::default();
    imgproc::cvt_color(&raw, &mut gray, imgproc::COLOR_BGR2GRAY, 0).unwrap();

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

// default threshold = 500
fn check_difference(img1: &Mat, img2: &Mat, threshold: i32) -> bool {
    let mut gray1 = Mat::default();
    let mut gray2 = Mat::default();
    imgproc::cvt_color(&img1, &mut gray1, imgproc::COLOR_BGR2GRAY, 0).unwrap();
    imgproc::cvt_color(&img2, &mut gray2, imgproc::COLOR_BGR2GRAY, 0).unwrap();

    let cell_w = gray1.cols() / 8;
    let cell_h = gray1.rows() / 8;

    for row in 0..8 {
        for col in 0..8 {
            let roi = Rect::new(col * cell_w, row * cell_h, cell_w, cell_h);
            let patch1 = Mat::roi(&gray1, roi).unwrap();
            let patch2 = Mat::roi(&gray2, roi).unwrap();

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

fn main() {
    let _take_screenshot = false;
    let _extract_pieces = false;

    let start = Instant::now();

    let board = if _take_screenshot {
        println!("Loaded pieces in: {:?}", start.elapsed());
        let monitor = select_primary_monitor(false);
        println!("Monitor selection took: {:?}", start.elapsed());
        let monitor = monitor.expect("No primary monitor found");

        let raw = capture_entire_screen(&monitor);
        let dyn_image = DynamicImage::ImageRgba8(raw.clone());
        println!("Captured screen in: {:?}", start.elapsed());
        let rat = dynamic_image_to_mat(&dyn_image).unwrap();
        // img_proc::show(&rat, false).unwrap();
        let coords = get_board_region(&rat);

        let cropped = imageops::crop_imm(&raw, coords.0, coords.1, coords.2, coords.3).to_image();
        let dynimage = DynamicImage::ImageRgba8(cropped);
        dynamic_image_to_mat(&dynimage).unwrap()
    } else {
        opencv::imgcodecs::imread("board.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap()
    };
    // img_proc::show(&board, false).unwrap();

    if _extract_pieces {
        extract_pieces(&board).unwrap();
    }

    let mut pieces: std::collections::HashMap<char, Mat> = std::collections::HashMap::new();

    for &symbol in &['k', 'q', 'b', 'p', 'r', 'n', 'K', 'Q', 'B', 'P', 'R', 'N'] {
        let path = format!("pieces/{}.png", symbol);
        let mat = opencv::imgcodecs::imread(&path, opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
        pieces.insert(symbol, mat);
    }

    let mut gray_board = Mat::default();
    imgproc::cvt_color(&board, &mut gray_board, imgproc::COLOR_BGR2GRAY, 0).unwrap();
    let mut bin_board = Mat::default();
    imgproc::threshold(
        &gray_board,
        &mut bin_board,
        127.0,
        255.0,
        imgproc::THRESH_BINARY,
    )
    .unwrap();

    // TODO! remove clones
    let arr = [
        (pieces[&'P'].clone(), 0.1, 'P'),
        (pieces[&'p'].clone(), 0.1, 'p'),
        (pieces[&'B'].clone(), 0.1, 'B'),
        (pieces[&'b'].clone(), 0.1, 'b'),
        (pieces[&'r'].clone(), 0.1, 'r'),
        (pieces[&'R'].clone(), 0.1, 'R'),
        (pieces[&'n'].clone(), 0.1, 'n'),
        (pieces[&'N'].clone(), 0.1, 'N'),
        (pieces[&'q'].clone(), 0.1, 'q'),
        (pieces[&'Q'].clone(), 0.1, 'Q'),
        (pieces[&'k'].clone(), 0.1, 'k'),
        (pieces[&'K'].clone(), 0.1, 'K'),
    ];

    let result = Arc::new(Mutex::new([[' '; 8]; 8]));
    let bin_board = Arc::new(bin_board);

    let s = Instant::now();

    let mut handles = vec![];
    for (piece, thres, sign) in arr {
        let board = Arc::clone(&bin_board);
        let result_ref = Arc::clone(&result);

        let handle = thread::spawn(move || {
            let local_result = img_proc::single_process(&board, &piece, thres, sign).unwrap();

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

    engine::Board::new(*result.lock().unwrap()).print();
    println!("Processing took: {:?}", s.elapsed());
}

fn extract_pieces(img: &Mat) -> Result<()> {
    let board_size = img.rows().min(img.cols());
    let board_size_f = board_size as f32;

    let mut x_edges = vec![];
    let mut y_edges = vec![];

    for i in 0..=8 {
        x_edges.push(((i as f32) * board_size_f / 8.0).round() as i32);
        y_edges.push(((i as f32) * board_size_f / 8.0).round() as i32);
    }

    let named_fields = vec![
        ((0, 0), "r"),
        ((1, 0), "n"),
        ((2, 0), "b"),
        ((3, 0), "q"),
        ((4, 0), "k"),
        ((0, 1), "p"),
        ((0, 6), "P"),
        ((0, 7), "R"),
        ((1, 7), "N"),
        ((2, 7), "B"),
        ((3, 7), "Q"),
        ((4, 7), "K"),
    ];

    for ((col, row), name) in named_fields {
        let x = x_edges[col];
        let y = y_edges[row];
        let w = x_edges[col + 1] - x;
        let h = y_edges[row + 1] - y;

        // TODO!
        let margin = 5;
        let x = x + margin;
        let y = y + margin;
        let w = (w - 2 * margin).max(1);
        let h = (h - 2 * margin).max(1);

        let roi = Rect::new(x, y, w, h);

        let img = Mat::roi(img, roi)?;

        let mut thresholded = Mat::default();
        imgproc::cvt_color(&img, &mut thresholded, imgproc::COLOR_BGR2GRAY, 0)?;
        let mut bin_board = Mat::default();
        imgproc::threshold(
            &thresholded,
            &mut bin_board,
            127.0,
            255.0,
            imgproc::THRESH_BINARY,
        )?;

        opencv::imgcodecs::imwrite(
            &format!("pieces/{name}.png"),
            &bin_board,
            &opencv::core::Vector::<i32>::new(),
        )?;
    }
    Ok(())
}

fn dynamic_image_to_mat(img: &DynamicImage) -> opencv::Result<Mat> {
    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();

    let mut mat =
        Mat::new_rows_cols_with_default(height as i32, width as i32, CV_8UC4, Scalar::all(0.0))
            .unwrap();

    let mat_data = mat.data_bytes_mut().unwrap();
    mat_data.copy_from_slice(&rgba8.as_raw());

    let mut mat_bgra = Mat::default();
    imgproc::cvt_color(&mat, &mut mat_bgra, imgproc::COLOR_RGBA2BGRA, 0).unwrap();
    Ok(mat_bgra)
}
