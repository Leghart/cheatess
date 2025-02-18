use image::{ImageBuffer, Rgba};
use opencv::{
    core::{min_max_loc, split, Mat, Point, Rect, Scalar, Size, Vector, CV_8UC4},
    highgui::{self, destroy_window},
    imgcodecs, imgproc,
    prelude::*,
};

pub struct ImageProcessing {}

impl ImageProcessing {
    pub fn read(path: &str) -> Result<Mat, Box<dyn std::error::Error>> {
        Ok(imgcodecs::imread(path, imgcodecs::IMREAD_UNCHANGED)?)
    }
    pub fn write(path: &str, image: &Mat) -> Result<bool, Box<dyn std::error::Error>> {
        let params = Vector::from_iter([0, 16]);
        Ok(imgcodecs::imwrite(path, &image, &params)?)
    }

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

    pub fn image_buffer_to_mat(
        buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    ) -> Result<Mat, Box<dyn std::error::Error>> {
        let (width, height) = buffer.dimensions();
        let mut mat = Mat::new_rows_cols_with_default(
            height as i32,
            width as i32,
            CV_8UC4,
            Scalar::all(0.0),
        )?;

        let mat_data = mat.data_bytes_mut()?;
        mat_data.copy_from_slice(&buffer.as_raw());

        let mut mat_bgr = Mat::default();
        imgproc::cvt_color(&mat, &mut mat_bgr, imgproc::COLOR_RGBA2BGR, 0)?;

        Ok(mat_bgr)
    }

    pub fn draw_box(
        image: &mut Mat,
        start_point: &Point,
        size: &Size,
        color: Scalar,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rect = Rect::new(start_point.x, start_point.y, size.width, size.height);
        imgproc::rectangle(image, rect, color, 2, 8, 0)?;
        Ok(())
    }

    // pub fn put_text() -> Result<(), Box<dyn std::error::Error>> {}

    pub fn threshold(image: &Mat) -> Result<Mat, Box<dyn std::error::Error>> {
        let mut thresholded = Mat::default();
        imgproc::cvt_color(&image, &mut thresholded, imgproc::COLOR_BGR2GRAY, 0)?;

        Ok(thresholded)
    }

    pub fn match_template(
        image_bg: &Mat,
        image_fg: &Mat,
    ) -> Result<Mat, Box<dyn std::error::Error>> {
        let mask = ImageProcessing::get_mask(&image_fg)?;
        let mut matched = Mat::default();

        imgproc::match_template(
            &image_bg,
            &image_fg,
            &mut matched,
            imgproc::TM_SQDIFF_NORMED,
            &mask,
        )?;
        Ok(matched)
    }

    fn get_mask(image: &Mat) -> Result<Mat, Box<dyn std::error::Error>> {
        let mut mask = Mat::default();
        let mut channels: Vector<Mat> = Vector::new();
        split(&image, &mut channels)?;
        mask = channels.get(3)?;

        Ok(mask)
    }
}

fn pixels_to_board(point: Point, size: &Size, piece: &char, board: &mut [[char; 8]; 8]) {
    let tile_width = size.width / 8;
    let tile_height = size.height / 8;

    let col = (point.x / tile_width).clamp(0, 7);
    let row = (7 - (point.y / tile_height)).clamp(0, 7);

    board[row as usize][col as usize] = *piece;
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
