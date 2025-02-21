use opencv::{
    core::{min_max_loc, Mat, Point, Rect, Scalar},
    imgproc,
    prelude::*,
};

use screenshots::Screen;
use std::collections::HashMap;
use std::fs;

pub mod chesscom;
pub mod lichess;
use super::engine::register_piece;
use super::image::ImageProcessing;

pub trait ChessboardTrackerInterface: Default {
    fn r#type(&self) -> WrapperType;

    fn get_region(&self) -> &Rect;

    fn get_thresholds(&self) -> &HashMap<String, f64>;

    fn pieces_path(&self) -> &'static str;

    fn capture_screenshot(&self) -> Result<Mat, Box<dyn std::error::Error>> {
        let region = self.get_region();
        let screen = Screen::all()?.first().unwrap().capture_area(
            region.x,
            region.y,
            region.width as u32,
            region.height as u32,
        )?;

        Ok(ImageProcessing::image_buffer_to_mat(screen)?)
    }

    fn load_pieces(&self) -> Result<HashMap<String, (Mat, f64)>, Box<dyn std::error::Error>> {
        let mut pieces: HashMap<String, (Mat, f64)> = HashMap::new();
        let thresholds = self.get_thresholds();
        let path_str = self.pieces_path();

        for entry in fs::read_dir(format!("../pieces/{path_str}/")).unwrap() {
            if let Ok(entry) = entry {
                let file_name = entry
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                let img = ImageProcessing::read(&entry.path().to_str().unwrap())?;
                pieces.insert(
                    file_name.clone(),
                    (img, *thresholds.get(&file_name).unwrap()),
                );
            }
        }

        Ok(pieces)
    }

    fn process_image(
        &self,
        board_image: &Mat,
        pieces: HashMap<String, (Mat, f64)>,
    ) -> Result<[[char; 8]; 8], Box<dyn std::error::Error>> {
        let mut result: [[char; 8]; 8] = [[' '; 8]; 8];

        for piece_name in pieces.keys() {
            let piece_threshold = pieces.get(piece_name).unwrap().clone().1;
            let mut piece_image = pieces.get(piece_name).unwrap().clone().0;

            if self.pieces_path() == "chesscom" && *piece_name == "p".to_string() {
                piece_image = ImageProcessing::resize(&piece_image, 43, 43).unwrap();
            }

            let board_gray = ImageProcessing::threshold(&board_image)?;
            let piece_gray = ImageProcessing::threshold(&piece_image)?;

            let mask = ImageProcessing::get_mask(&piece_image)?;
            let mut matched = ImageProcessing::match_template(&board_gray, &piece_gray, &mask)?;

            let mut min_val = 0.0;
            let mut max_val = 0.0;
            let mut min_loc = Point::default();
            let mut max_loc = Point::default();

            let board_size = board_gray.size().unwrap();

            min_max_loc(
                &matched,
                Some(&mut min_val),
                Some(&mut max_val),
                Some(&mut min_loc),
                Some(&mut max_loc),
                &Mat::default(),
            )?;

            while min_val < piece_threshold {
                let top_left = min_loc;

                register_piece(
                    (top_left.x, top_left.y),
                    (board_size.width, board_size.height),
                    &piece_name.chars().next().unwrap(),
                    &mut result,
                );

                let size = matched.size()?;
                let top_x = top_left.x.max(0).min(size.width - 1);
                let top_y = top_left.y.max(0).min(size.height - 1);

                let rect_x = (top_x as i32 - 22).max(0);
                let rect_y = (top_y as i32 - 22).max(0);

                let rect_w = (45).min(size.width - rect_x);
                let rect_h = (45).min(size.height - rect_y);

                let poison = Rect::new(rect_x, rect_y, rect_w, rect_h);

                let mut result_slice = matched.roi_mut(poison)?;
                result_slice
                    .set_to(&Scalar::all(1.0), &Mat::default())
                    .unwrap();

                min_max_loc(
                    &matched,
                    Some(&mut min_val),
                    Some(&mut max_val),
                    Some(&mut min_loc),
                    Some(&mut max_loc),
                    &Mat::default(),
                )
                .unwrap();
            }
        }

        Ok(result)
    }

    // TODO: reuse process_image
    fn visualize_process_image(
        &self,
        board_image: &Mat,
        pieces: HashMap<String, (Mat, f64)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut board_clone = board_image.clone();

        for piece_name in pieces.keys() {
            let piece_threshold = pieces.get(piece_name).unwrap().clone().1;
            let mut piece_image = pieces.get(piece_name).unwrap().clone().0;

            if self.pieces_path() == "chesscom" && *piece_name == "p".to_string() {
                piece_image = ImageProcessing::resize(&piece_image, 43, 43).unwrap();
            }

            let board_gray = ImageProcessing::threshold(&board_image)?;
            let piece_gray = ImageProcessing::threshold(&piece_image)?;

            let size = piece_gray.size().unwrap();
            let (h, w) = (size.height, size.width);

            let mask = ImageProcessing::get_mask(&piece_image)?;
            let mut matched = ImageProcessing::match_template(&board_gray, &piece_gray, &mask)?;

            let mut min_val = 0.0;
            let mut max_val = 0.0;
            let mut min_loc = Point::default();
            let mut max_loc = Point::default();

            min_max_loc(
                &matched,
                Some(&mut min_val),
                Some(&mut max_val),
                Some(&mut min_loc),
                Some(&mut max_loc),
                &Mat::default(),
            )?;

            while min_val < piece_threshold {
                let top_left = min_loc;

                let rectangle_color = Scalar::new(0.0, 250.0, 50.0, 0.0);
                let rect = Rect::new(top_left.x, top_left.y, w, h);
                imgproc::rectangle(&mut board_clone, rect, rectangle_color, 2, 8, 0)?;

                let text_color = if piece_name.chars().next().unwrap().is_uppercase() {
                    Scalar::new(255.0, 0.0, 0.0, 0.0)
                } else {
                    Scalar::new(0.0, 0.0, 255.0, 0.0)
                };

                let text_position = Point::new(top_left.x, top_left.y + 20);
                imgproc::put_text(
                    &mut board_clone,
                    &piece_name.to_string(),
                    text_position,
                    imgproc::FONT_HERSHEY_SIMPLEX,
                    0.7,
                    text_color,
                    2,
                    8,
                    false,
                )?;

                ImageProcessing::show(&board_clone, false)?;

                let size = matched.size()?;
                let top_x = top_left.x.max(0).min(size.width - 1);
                let top_y = top_left.y.max(0).min(size.height - 1);

                let rect_x = (top_x as i32 - 22).max(0);
                let rect_y = (top_y as i32 - 22).max(0);

                let rect_w = (45).min(size.width - rect_x);
                let rect_h = (45).min(size.height - rect_y);

                let poison = Rect::new(rect_x, rect_y, rect_w, rect_h);

                let mut result_slice = matched.roi_mut(poison)?;
                result_slice
                    .set_to(&Scalar::all(1.0), &Mat::default())
                    .unwrap();

                min_max_loc(
                    &matched,
                    Some(&mut min_val),
                    Some(&mut max_val),
                    Some(&mut min_loc),
                    Some(&mut max_loc),
                    &Mat::default(),
                )
                .unwrap();
            }
        }

        Ok(())
    }
}

pub enum WrapperType {
    Chesscom,
    Lichess,
}
