extern crate opencv;

use opencv::core::Rect;
use opencv::{
    core::{min_max_loc, split, Mat, Point, Scalar, Vector},
    imgcodecs, imgproc,
    prelude::*,
};
use std::collections::HashMap;
use std::time::Instant;

const CHESS_BOARD_OUTPUT_DIR: &str = "dist/";

const EXPORT_IMAGE: bool = true;

fn main() -> opencv::Result<()> {
    let chess_piece_threshold: HashMap<char, f64> = [
        ('B', 0.2),
        ('b', 0.3),
        ('K', 0.2),
        ('k', 0.2),
        ('N', 0.1),
        ('n', 0.2),
        ('P', 0.15),
        ('p', 0.3),
        ('Q', 0.2),
        ('q', 0.2),
        ('R', 0.2),
        ('r', 0.2),
    ]
    .iter()
    .cloned()
    .collect();

    let chess_piece_images = load_chess_piece_images(&chess_piece_threshold)?;
    let chess_board_images = load_chess_board_images()?;

    let st = Instant::now();

    for (board_name, board_image) in chess_board_images {
        detect_piece_of_chess(&board_name, &board_image, &chess_piece_images)?;
    }

    println!("Execution time: {:?}", st.elapsed());
    Ok(())
}

fn load_chess_piece_images(
    chess_piece_threshold: &HashMap<char, f64>,
) -> opencv::Result<HashMap<char, (Mat, f64)>> {
    let mut chess_piece_images = HashMap::new();

    for entry in std::fs::read_dir("chess_piece").unwrap() {
        let path = entry.unwrap().path();
        let base_name = path.file_name().unwrap().to_str().unwrap();
        let piece_name = base_name.chars().next().unwrap();

        let piece_image = imgcodecs::imread(path.to_str().unwrap(), imgcodecs::IMREAD_UNCHANGED)?;
        if let Some(&threshold) = chess_piece_threshold.get(&piece_name) {
            chess_piece_images.insert(piece_name, (piece_image, threshold));
        }
    }
    Ok(chess_piece_images)
}

fn load_chess_board_images() -> opencv::Result<HashMap<String, Mat>> {
    let mut chess_board_images = HashMap::new();
    for entry in std::fs::read_dir("test").unwrap() {
        let path = entry.unwrap().path();
        let base_name = path.file_name().unwrap().to_str().unwrap();
        let board_image = imgcodecs::imread(path.to_str().unwrap(), imgcodecs::IMREAD_COLOR)?;
        chess_board_images.insert(base_name.to_string(), board_image);
    }
    Ok(chess_board_images)
}

fn detect_piece_of_chess(
    board_name: &str,
    board_image: &Mat,
    chess_piece_images: &HashMap<char, (Mat, f64)>,
) -> opencv::Result<()> {
    let mut result_image = board_image.clone();

    for (piece_name, (piece_image, threshold)) in chess_piece_images {
        let mut piece_image_gray = Mat::default();
        imgproc::cvt_color(
            &piece_image,
            &mut piece_image_gray,
            imgproc::COLOR_BGR2GRAY,
            0,
        )?;

        let mut board_image_gray = Mat::default();
        imgproc::cvt_color(
            &result_image,
            &mut board_image_gray,
            imgproc::COLOR_BGR2GRAY,
            0,
        )?;

        let mut mask = Mat::default();
        let mut channels: Vector<Mat> = Vector::new();
        if piece_image.channels() == 4 {
            split(&piece_image, &mut channels)?;
            mask = channels.get(3)?;
        }

        let size = piece_image_gray.size()?;
        let (h, w) = (size.height, size.width);

        let mut result = Mat::default();
        imgproc::match_template(
            &board_image_gray,
            &piece_image_gray,
            &mut result,
            imgproc::TM_SQDIFF_NORMED,
            &mask,
        )?;

        let mut min_val = 0.0;
        let mut max_val = 0.0;
        let mut min_loc = Point::default();
        let mut max_loc = Point::default();

        let mask = Mat::default();
        min_max_loc(
            &result,
            Some(&mut min_val),
            Some(&mut max_val),
            Some(&mut min_loc),
            Some(&mut max_loc),
            &mask,
        )?;

        while min_val < *threshold {
            let top_left = min_loc;

            // Draw the rectangle around the detected piece
            let rectangle_color = Scalar::new(0.0, 250.0, 50.0, 0.0); // Highlight with green
            let rect = Rect::new(top_left.x, top_left.y, w, h);
            imgproc::rectangle(&mut result_image, rect, rectangle_color, 2, 8, 0)?; //ok

            // Choose text color depending on whether it's a black or white piece
            let text_color = if piece_name.is_uppercase() {
                Scalar::new(255.0, 0.0, 0.0, 0.0) // Red for black pieces
            } else {
                Scalar::new(0.0, 0.0, 255.0, 0.0) // Blue for white pieces
            };

            let text_position = Point::new(top_left.x, top_left.y + 20);
            imgproc::put_text(
                &mut result_image,
                &piece_name.to_string(),
                text_position,
                imgproc::FONT_HERSHEY_SIMPLEX,
                0.7,
                text_color,
                2,
                8,
                false,
            )?;

            // Overwrite the result matrix in the vicinity of this match to avoid re-detection
            let mut h1 = top_left.y - h / 2;
            h1 = h1.max(0).min(result.rows() - 1);

            let mut h2 = top_left.y + h / 2 + 1;
            h2 = h2.max(0).min(result.rows() - 1);

            let mut w1 = top_left.x - w / 2;
            w1 = w1.max(0).min(result.cols() - 1);

            let mut w2 = top_left.x + w / 2 + 1;
            w2 = w2.max(0).min(result.cols() - 1);

            //ok

            let copy_r = result.clone();
            // Poison the result to prevent finding the same match again
            let mut result_slice = result.roi_mut(Rect::new(w1, h1, w2 - w1, h2 - h1))?;
            result_slice.set_to(&Scalar::new(1.0, 0.0, 0.0, 0.0), &Mat::default())?;
            // let a = *result_slice;

            assert_ne!(copy_r.data_bytes()?, result.data_bytes()?);
            // Find next match
            min_max_loc(
                &result,
                Some(&mut min_val),
                Some(&mut max_val),
                Some(&mut min_loc),
                Some(&mut max_loc),
                &Mat::default(),
            )?;
        }
    }

    if EXPORT_IMAGE {
        let params = Vector::from_iter([0, 16]);
        imgcodecs::imwrite(
            &format!("{}/{}", CHESS_BOARD_OUTPUT_DIR, board_name),
            &result_image,
            &params,
        )?;
    }

    Ok(())
}
