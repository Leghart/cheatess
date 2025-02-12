mod utils;

extern crate image;
extern crate opencv;

use opencv::{
    core::{Mat, Vector, CV_8UC1},
    highgui, imgcodecs, imgproc,
    objdetect::CascadeClassifier,
    prelude::*,
};

fn main() {
    let screen_area = utils::get_screen_area().unwrap();
    let image = utils::take_screenshot(&screen_area).unwrap();
    let mat = utils::to_mat(&image);
    let binary = utils::to_binary(&mat);
    let resized = utils::resize(&binary, 200, 200).unwrap();
}
