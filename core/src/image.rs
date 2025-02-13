use opencv::{
    core::{min_max_loc, split, Mat, Point, Rect, Scalar, Size, Vector},
    imgcodecs, imgproc,
    prelude::*,
};
use std::collections::HashMap;

pub struct ImageProcessing {
    pieces_images: HashMap<char, (Mat, f64)>,
    pieces_thresholds: HashMap<char, f64>,
}

impl ImageProcessing {
    // pub fn new() -> Self {
    //     ImageProcessing {
    //         pieces_images: HashMap::new(),
    //         pieces_thresholds: HashMap::new(),
    //     }
    // }

    pub fn default() -> Self {
        let pieces_thresholds: HashMap<char, f64> = [
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
        ImageProcessing {
            pieces_thresholds,
            pieces_images: HashMap::new(),
        }
    }

    //     Scalar::new(255.0, 0.0, 0.0, 0.0) // Red for black pieces
    //     Scalar::new(0.0, 0.0, 255.0, 0.0) // Blue for white pieces
    fn draw_box(
        &self,
        image: &mut Mat,
        start_point: &Point,
        size: &Size,
        color: Scalar,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rect = Rect::new(start_point.x, start_point.y, size.width, size.height);
        imgproc::rectangle(image, rect, color, 2, 8, 0)?;
        Ok(())
    }

    fn threshold_image(&self, image: &Mat) -> Result<Mat, Box<dyn std::error::Error>> {
        let mut thresholded = Mat::default();
        imgproc::cvt_color(&image, &mut thresholded, imgproc::COLOR_BGR2GRAY, 0)?;
        Ok(thresholded)
    }

    fn get_mask(&self, image: &Mat) -> Result<Mat, Box<dyn std::error::Error>> {
        let mut mask = Mat::default();
        let mut channels: Vector<Mat> = Vector::new();
        if image.channels() == 4 {
            split(&image, &mut channels)?;
            mask = channels.get(3)?;
        }
        Ok(mask)
    }

    fn save_image(&self, image: &Mat, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let params = Vector::from_iter([0, 16]);
        imgcodecs::imwrite(path, &image, &params)?;
        Ok(())
    }

    fn set_pieces_thresholds(&mut self, pieces_thresholds: HashMap<char, f64>) {
        self.pieces_thresholds = pieces_thresholds;
    }

    pub fn load_piece_images(&mut self, dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        assert_ne!(self.pieces_thresholds.len(), 0);
        // TODO: change mapping name of pieces
        let mut chess_piece_images = HashMap::new();

        for entry in std::fs::read_dir(dir_path).unwrap() {
            let path = entry.unwrap().path();
            let base_name = path.file_name().unwrap().to_str().unwrap();
            let piece_name = base_name.chars().next().unwrap();

            let piece_image =
                imgcodecs::imread(path.to_str().unwrap(), imgcodecs::IMREAD_UNCHANGED)?;
            if let Some(&threshold) = self.pieces_thresholds.get(&piece_name) {
                chess_piece_images.insert(piece_name, (piece_image, threshold));
            }
        }
        self.pieces_images = chess_piece_images;
        Ok(())
    }

    fn load_pieces_thresholds(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn load_image(&self, path: &str) -> Result<Mat, Box<dyn std::error::Error>> {
        let image = imgcodecs::imread(path, imgcodecs::IMREAD_COLOR)?;
        Ok(image)
    }

    pub fn image_to_board(
        &self,
        image: &Mat,
    ) -> Result<[[char; 8]; 8], Box<dyn std::error::Error>> {
        let mut board: [[char; 8]; 8] = [[' '; 8]; 8];

        for (piece_name, (piece_image, threshold)) in &self.pieces_images {
            let gray_piece = self.threshold_image(&piece_image)?;
            let gray_board = self.threshold_image(&image)?;
            let mask = self.get_mask(&piece_image)?;

            let size = gray_piece.size()?;
            let (h, w) = (size.height, size.width);

            let mut result = Mat::default();
            imgproc::match_template(
                &gray_board,
                &gray_piece,
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

            let mut ctr = 0;
            while min_val < *threshold {
                if ctr >= 10 {
                    break;
                }
                let top_left = min_loc;

                self.pixels_to_board(top_left, &result.size().unwrap(), piece_name, &mut board);

                let mut h1 = top_left.y - h / 2;
                h1 = h1.max(0).min(result.rows() - 1);

                let mut h2 = top_left.y + h / 2 + 1;
                h2 = h2.max(0).min(result.rows() - 1);

                let mut w1 = top_left.x - w / 2;
                w1 = w1.max(0).min(result.cols() - 1);

                let mut w2 = top_left.x + w / 2 + 1;
                w2 = w2.max(0).min(result.cols() - 1);

                let mut result_slice = result.roi_mut(Rect::new(w1, h1, w2 - w1, h2 - h1))?;
                result_slice.set_to(&Scalar::new(1.0, 0.0, 0.0, 0.0), &Mat::default())?;

                min_max_loc(
                    &result,
                    Some(&mut min_val),
                    Some(&mut max_val),
                    Some(&mut min_loc),
                    Some(&mut max_loc),
                    &Mat::default(),
                )?;
                ctr += 1;
            }
        }

        Ok(board)
    }

    fn pixels_to_board(&self, point: Point, size: &Size, piece: &char, board: &mut [[char; 8]; 8]) {
        let tile_width = size.width / 8;
        let tile_height = size.height / 8;

        let col = (point.x / tile_width).clamp(0, 7);
        let row = (7 - (point.y / tile_height)).clamp(0, 7);

        board[row as usize][col as usize] = *piece;
    }
}

fn pos_to_algebraic(row: usize, col: usize) -> String {
    let file = (b'a' + col as u8) as char;
    let rank = (8 - row).to_string();
    format!("{}{}", file, rank)
}

fn find_move(before: [[char; 8]; 8], after: [[char; 8]; 8]) -> Option<String> {
    let mut from: Option<(usize, usize)> = None;
    let mut to: Option<(usize, usize)> = None;

    for row in 0..8 {
        for col in 0..8 {
            if before[row][col] != after[row][col] {
                if before[row][col] != ' ' && after[row][col] == ' ' {
                    from = Some((row, col));
                }
                if before[row][col] == ' ' && after[row][col] != ' ' {
                    to = Some((row, col));
                }
            }
        }
    }

    if let (Some((from_row, from_col)), Some((to_row, to_col))) = (from, to) {
        Some(format!(
            "{}{}",
            pos_to_algebraic(from_row, from_col),
            pos_to_algebraic(to_row, to_col)
        ))
    } else {
        None
    }
}
