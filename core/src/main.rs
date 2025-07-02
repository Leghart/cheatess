mod engine;
mod img_proc;
mod stockfish;

use std::time::Instant;

use opencv::{
    core::{Mat, Point, Rect, Scalar, CV_8UC4},
    imgproc,
    prelude::*,
};
use std::sync::{Arc, Mutex};
use std::thread;

use image::{imageops, DynamicImage};

use image::ImageBuffer;
use image::Rgba;
use opencv::Result;
use screenshots::Screen;
use xcap::Monitor;


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

fn capture_entire_screen(monitor: &Monitor) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let capture = monitor.capture_image().unwrap();

    ImageBuffer::<Rgba<u8>, _>::from_raw(capture.width(), capture.height(), capture.into_vec())
        .unwrap()
}

// This function captures the screen and returns the region of the chessboard
// with the following steps:
// - Apply Canny edge detection to find the edges
// - Find contours in the edge-detected image
// - Approximate the contours to find quadrilaterals
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

// default threshold = 500
fn images_have_differences(gray1: &Mat, gray2: &Mat, threshold: i32) -> bool {
    let cell_w = gray1.cols() / 8;
    let cell_h = gray1.rows() / 8;
    // TODO!: to fix
    println!("Cell size: {}x{}", cell_w, cell_h); // Debugging output
    for row in 0..8 {
        for col in 0..8 {
            let x = col * cell_w;
            let y = row * cell_h;

            let width = if col == 7 { gray1.cols() - x } else { cell_w };
            let height = if row == 7 { gray1.rows() - y } else { cell_h };

            let roi = Rect::new(x, y, width, height);
            println!("ROI: {:?}", roi); // Debugging output
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

fn main() {
    let monitor = select_monitor(true).expect("No primary monitor found");

    // let raw = monitor.capture().unwrap();
    let raw = capture_entire_screen(&monitor);
    let dyn_image = DynamicImage::ImageRgba8(raw.clone());
    let base_gray_board = dynamic_image_to_gray_mat(&dyn_image).unwrap();

    let coords = get_board_region(&base_gray_board);

    let cropped = imageops::crop_imm(&raw, coords.0, coords.1, coords.2, coords.3).to_image();
    let dyn_image = DynamicImage::ImageRgba8(cropped.clone());
    let board = dynamic_image_to_gray_mat(&dyn_image).unwrap();
    let pieces = extract_pieces(&board).unwrap();

    let pieces: Arc<Vec<(char, Arc<Mat>)>> = Arc::new(
        pieces
            .into_iter()
            .map(|(sign, piece)| (sign, Arc::new(piece)))
            .collect(),
    );

    loop {
        let start = Instant::now();
        let raw = capture_entire_screen(&monitor);

        println!("Captured screen in: {:?}", start.elapsed());
        let cropped = imageops::crop_imm(&raw, coords.0, coords.1, coords.2, coords.3).to_image();
        let dyn_image = DynamicImage::ImageRgba8(cropped.clone());
        let gray_board = dynamic_image_to_gray_mat(&dyn_image).unwrap();

        // if images_have_differences(&base_gray_board, &gray_board, 500) {
        //     println!("Board changed, processing...");
        // } else {
        //     println!("No changes detected, skipping processing.");
        //     continue;
        // }

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

        println!("Board processed in: {:?}", start.elapsed());
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
        println!("All: {:?}", start.elapsed());

        engine::Board::new(*result.lock().unwrap()).print();
    }
}

fn extract_pieces(
    img: &Mat,
) -> Result<std::collections::HashMap<char, Mat>, Box<dyn std::error::Error>> {
    let board_size = img.rows().min(img.cols());
    let board_size_f = board_size as f32;

    let mut x_edges = vec![];
    let mut y_edges = vec![];

    for i in 0..=8 {
        x_edges.push(((i as f32) * board_size_f / 8.0).round() as i32);
        y_edges.push(((i as f32) * board_size_f / 8.0).round() as i32);
    }

    let named_fields = vec![
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

    let mut result = std::collections::HashMap::new();
    for ((col, row), name) in named_fields {
        let x = x_edges[col];
        let y = y_edges[row];
        let w = x_edges[col + 1] - x;
        let h = y_edges[row + 1] - y;

        // Add a margin to the piece extraction area
        let margin = 5;
        let x = x + margin;
        let y = y + margin;
        let w = (w - 2 * margin).max(1);
        let h = (h - 2 * margin).max(1);

        let roi = Rect::new(x, y, w, h);
        let img = Mat::roi(img, roi)?;

        let mut bin_board = Mat::default();
        imgproc::threshold(&img, &mut bin_board, 127.0, 255.0, imgproc::THRESH_BINARY)?;
        result.insert(name, bin_board);
    }
    Ok(result)
}

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
