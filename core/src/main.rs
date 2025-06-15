mod config;
mod engine;
mod myimage;
mod stockfish;
mod utils;

pub mod webwrapper;
// use image::ImageProcessing;
use opencv::imgcodecs;
use std::time::Instant;
use webwrapper::chesscom::ChesscomWrapper;
use webwrapper::ChessboardTrackerInterface;

use crossbeam::channel::{bounded, Receiver, Sender};
use opencv::{
    core::{split, Mat, Point, Rect, Scalar, Size, Vector, CV_8UC3, CV_8UC4},
    highgui::{self, destroy_window},
    imgproc,
    prelude::*,
};
use std::sync::{Arc, Mutex};
use std::thread;

use image::{DynamicImage, GenericImageView};
use opencv::{core::Mat_AUTO_STEP, prelude::*};
use xcap::Monitor;

use std::io::Read;
use std::process::{Command, Stdio};

use image::{imageops, ImageBuffer, Rgba};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use opencv::{prelude::*, types, Result};

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

fn main() {
    let monitor = select_primary_monitor(false);

    let monitor = monitor.expect("No primary monitor found");
    let raw = capture_entire_screen(&monitor);
    let dyn_image = DynamicImage::ImageRgba8(raw.clone());

    let rat = dynamic_image_to_mat(&dyn_image).unwrap();
    let coords = get_board_region(&rat);

    let cropped = imageops::crop_imm(&raw, coords.0, coords.1, coords.2, coords.3).to_image();
    let dynimage = DynamicImage::ImageRgba8(cropped);
    let board = dynamic_image_to_mat(&dynimage).unwrap();
    // myimage::ImageProcessing::show(&board, true).unwrap();
    extract_pieces(&board).unwrap();

    let piece1 =
        opencv::imgcodecs::imread("pieces/k.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
    let piece2 =
        opencv::imgcodecs::imread("pieces/q.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
    let piece3 =
        opencv::imgcodecs::imread("pieces/b.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
    let piece4 =
        opencv::imgcodecs::imread("pieces/p.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
    let piece5 =
        opencv::imgcodecs::imread("pieces/r.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
    let piece6 =
        opencv::imgcodecs::imread("pieces/n.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();

    let piece11 =
        opencv::imgcodecs::imread("pieces/K.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
    let piece22 =
        opencv::imgcodecs::imread("pieces/Q.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
    let piece33 =
        opencv::imgcodecs::imread("pieces/B.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
    let piece44 =
        opencv::imgcodecs::imread("pieces/P.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
    let piece55 =
        opencv::imgcodecs::imread("pieces/R.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();
    let piece66 =
        opencv::imgcodecs::imread("pieces/N.png", opencv::imgcodecs::IMREAD_UNCHANGED).unwrap();

    let tracker = ChesscomWrapper::default();
    let r = tracker
        .process_image(
            &board,
            &std::collections::HashMap::from_iter([
                ('k'.to_string(), (piece1, 0.1)),
                ('q'.to_string(), (piece2, 0.1)),
                ('b'.to_string(), (piece3, 0.1)),
                ('p'.to_string(), (piece4, 0.4)),
                ('r'.to_string(), (piece5, 0.2)),
                ('n'.to_string(), (piece6, 0.1)),
                ('K'.to_string(), (piece11, 0.1)),
                ('Q'.to_string(), (piece22, 0.1)),
                ('B'.to_string(), (piece33, 0.1)),
                ('P'.to_string(), (piece44, 0.2)),
                ('R'.to_string(), (piece55, 0.2)),
                ('N'.to_string(), (piece66, 0.1)),
            ]),
        )
        .unwrap();
    let board = engine::Board::new(r);
    board.print();
}

fn extract_pieces(img: &Mat) -> Result<()> {
    // let board_size = img.rows().min(img.cols());
    // let square_size = board_size as f32 / 8.0;

    let board_size = img.rows().min(img.cols()); // załóżmy kwadratowa
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
        // let y_start = (row as f32 * square_size).round() as i32;
        // let y_end = ((row + 1) as f32 * square_size).round() as i32;
        // let x_start = (col as f32 * square_size).round() as i32;
        // let x_end = ((col + 1) as f32 * square_size).round() as i32;

        // let roi = Rect::new(
        //     x_start,
        //     y_start,
        //     (x_end - x_start).max(1),
        //     (y_end - y_start).max(1),
        // );

        let x = x_edges[col];
        let y = y_edges[row];
        let w = x_edges[col + 1] - x;
        let h = y_edges[row + 1] - y;

        let roi = Rect::new(x, y, w, h);

        let img = Mat::roi(img, roi)?;

        let mut img_bgra = Mat::default();
        opencv::imgproc::cvt_color(&img, &mut img_bgra, opencv::imgproc::COLOR_BGR2BGRA, 0)?;

        let mut img_bgr = Mat::default();
        opencv::imgproc::cvt_color(&img_bgra, &mut img_bgr, opencv::imgproc::COLOR_BGRA2BGR, 0)?;

        let offset = 6;

        let probe_y = if row % 2 == 0 { offset } else { h - 1 - offset };

        let probe_x = if col % 2 == 0 { offset } else { w - 1 - offset };
        let px = img_bgr.at_2d::<opencv::core::Vec3b>(probe_y, probe_x)?;

        let base_b = px[0] as f64;
        let base_g = px[1] as f64;
        let base_r = px[2] as f64;
        let tol = 10.0;

        let lower = Scalar::new(
            (base_b - tol).max(0.0),
            (base_g - tol).max(0.0),
            (base_r - tol).max(0.0),
            0.0,
        );
        let upper = Scalar::new(
            (base_b + tol).min(255.0),
            (base_g + tol).min(255.0),
            (base_r + tol).min(255.0),
            0.0,
        );

        let mut mask = Mat::default();
        opencv::core::in_range(&img_bgr, &lower, &upper, &mut mask)?;

        // set aplha=0 to disable the background
        for y in 0..img_bgra.rows() {
            for x in 0..img_bgra.cols() {
                if *mask.at_2d::<u8>(y, x)? > 0 {
                    let pixel = img_bgra.at_2d_mut::<opencv::core::Vec4b>(y, x)?;
                    pixel[3] = 0;
                }
            }
        }

        // TEST:
        // let mut blurred = Mat::default();
        // imgproc::gaussian_blur(
        //     &img_bgra,
        //     &mut blurred,
        //     opencv::core::Size::new(3, 3),
        //     0.0,
        //     0.0,
        //     opencv::core::BORDER_DEFAULT,
        // )?;

        opencv::imgcodecs::imwrite(
            &format!("pieces/{name}.png"),
            &img_bgra,
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

fn process_image(image: &Mat) -> Result<[[char; 8]; 8], Box<dyn std::error::Error>> {
    Ok([['.'; 8]; 8])
}
