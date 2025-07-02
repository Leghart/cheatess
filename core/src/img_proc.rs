use super::engine::register_piece;

use opencv::{
    core::{min_max_loc, Mat, Point, Rect, Scalar},
    highgui::{self, destroy_window},
    imgproc,
    prelude::*,
};

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

/// Processes a single chess piece image against a board image.
/// This function uses template matching to find the piece on the board and registers its position.
/// It updates the `result` array with the found piece's symbol at the corresponding board position
/// Both images (board and piece) are already binary thresholded, so mask is not needed.
pub fn single_process(
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
            (top_left.x, top_left.y),
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
