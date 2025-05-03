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

fn main() {
    let (tx, rx): (Sender<Mat>, Receiver<Mat>) = bounded(2);
    let prev_hash = Arc::new(Mutex::new(0u64));

    let prev_hash_clone = Arc::clone(&prev_hash);
    thread::spawn(move || {
        let monitor = Monitor::from_point(100, 100).unwrap();
        loop {
            let st = Instant::now();
            if let Ok(capture) = monitor.capture_image() {
                let raw = ImageBuffer::<Rgba<u8>, _>::from_raw(
                    capture.width(),
                    capture.height(),
                    capture.into_vec(),
                )
                .unwrap();

                let cropped = imageops::crop_imm(&raw, 100, 100, 360, 360).to_image();
                let dynimage = DynamicImage::ImageRgba8(cropped);

                let hash = image_hash(&dynimage);
                let mut last_hash = prev_hash_clone.lock().unwrap();
                if *last_hash != hash {
                    *last_hash = hash;
                    if let Ok(mat) = dynamic_image_to_mat(&dynimage) {
                        tx.send(mat).ok();
                    }
                }
                print!("{:?}", st.elapsed());
            }
        }
    });

    while let Ok(mat) = rx.recv() {
        myimage::ImageProcessing::show(&mat, false).unwrap();
        // break;

        // if let Ok(board_data) = process_image(&mat) {
        //     println!("Got board data: {:?}", board_data);
        // }
    }
}

fn dynamic_image_to_mat(img: &DynamicImage) -> opencv::Result<Mat> {
    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();

    let mut mat =
        Mat::new_rows_cols_with_default(height as i32, width as i32, CV_8UC4, Scalar::all(0.0))?;

    let mat_data = mat.data_bytes_mut()?;
    mat_data.copy_from_slice(&rgba8.as_raw());

    let mut mat_bgr = Mat::default();
    imgproc::cvt_color(&mat, &mut mat_bgr, imgproc::COLOR_RGBA2BGR, 0)?;

    Ok(mat_bgr)
}

fn image_hash(img: &DynamicImage) -> u64 {
    let mut hasher = DefaultHasher::new();
    img.to_rgba8().as_raw().hash(&mut hasher);
    hasher.finish()
}

fn process_image(image: &Mat) -> Result<[[char; 8]; 8], Box<dyn std::error::Error>> {
    Ok([['.'; 8]; 8])
}

// fn _single_run() {
//     let total = std::time::Instant::now();
//     let st = std::time::Instant::now();
//     let tracker = ChesscomWrapper::default();
//     println!("tracker: {:?}", st.elapsed());

//     let st = std::time::Instant::now();
//     let image = tracker.capture_screenshot().unwrap();
//     println!("screenshot: {:?}", st.elapsed());

//     let st = std::time::Instant::now();
//     let resized = ImageProcessing::resize(&image, 360, 360).unwrap();
//     println!("resize: {:?}", st.elapsed());

//     let st = std::time::Instant::now();
//     let pieces = tracker.load_pieces().unwrap();
//     println!("laod pieces: {:?}", st.elapsed());

//     let st = std::time::Instant::now();
//     let _ = tracker.process_image(&resized, &pieces).unwrap();

//     println!("process: {:?}", st.elapsed());
//     println!("TOTOAL: {:?}", total.elapsed());

//     println!("\x1b[33m♕\x1b[0m");
// }

// fn _loop() {
//     let tracker_def = ChesscomWrapper::default();
//     // 440 219 758 759
//     // 445, 185, 742, 743
//     let tracker = ChesscomWrapper::new(
//         utils::screen_region::ScreenRegion::new(443, 183, 744, 745),
//         tracker_def.get_thresholds().clone(),
//     );
//     println!("tracker region: {:?}", tracker.get_region());
//     let pieces = tracker.load_pieces().unwrap();

//     loop {
//         let st = std::time::Instant::now();
//         // println!("start: {:?}", st.elapsed());

//         let image = tracker.capture_screenshot().unwrap();
//         // println!("screenshot: {:?}", st.elapsed());

//         let resized = ImageProcessing::resize(&image, 360, 360).unwrap();
//         // println!("resize: {:?}", st.elapsed());

//         let board_data = tracker.process_image(&resized, &pieces).unwrap();
//         // println!("process: {:?}", st.elapsed());

//         let board = engine::Board::new(board_data);
//         board.print();
//         // println!("{:?}", file_name);
//         // println!("cycle: {:?}", st.elapsed());
//         std::thread::sleep(std::time::Duration::from_millis(200));
//     }
// }
