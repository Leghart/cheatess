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
    myimage::ImageProcessing::show(&board, true).unwrap();
}

fn dynamic_image_to_mat(img: &DynamicImage) -> opencv::Result<Mat> {
    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();

    let mut mat =
        Mat::new_rows_cols_with_default(height as i32, width as i32, CV_8UC4, Scalar::all(0.0))
            .unwrap();

    let mat_data = mat.data_bytes_mut().unwrap();
    mat_data.copy_from_slice(&rgba8.as_raw());

    let mut mat_bgr = Mat::default();
    imgproc::cvt_color(&mat, &mut mat_bgr, imgproc::COLOR_RGBA2BGR, 0).unwrap();

    Ok(mat_bgr)
}

fn process_image(image: &Mat) -> Result<[[char; 8]; 8], Box<dyn std::error::Error>> {
    Ok([['.'; 8]; 8])
}
