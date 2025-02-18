use opencv::{
    core::{min_max_loc, split, KeyPoint, Mat, Point, Rect, Scalar, Size, Vector, CV_8UC4},
    features2d::{BFMatcher, ORB_ScoreType, ORB},
    highgui, imgcodecs, imgproc,
    prelude::*,
};
mod utils;
use screenshots::Screen;
use std::fs;
use std::{collections::HashMap, thread, time::Duration};

pub struct ChessboardTracker {
    region: Rect,
    thresholds: HashMap<String, f64>,
}

impl ChessboardTracker {
    pub fn new(region: Rect) -> Self {
        ChessboardTracker {
            region,
            thresholds: HashMap::from_iter([
                ("B".to_string(), 0.35),
                ("b".to_string(), 0.55),
                ("K".to_string(), 0.2),
                ("k".to_string(), 0.3),
                ("N".to_string(), 0.1),
                ("n".to_string(), 0.3),
                ("P".to_string(), 0.15),
                ("p".to_string(), 0.9),
                ("Q".to_string(), 0.7),
                ("q".to_string(), 0.3),
                ("R".to_string(), 0.4),
                ("r".to_string(), 0.3),
            ]),
        }
    }

    pub fn capture_screenshot(&self) -> Result<Mat, Box<dyn std::error::Error>> {
        let screen = Screen::all()?.first().unwrap().capture_area(
            self.region.x,
            self.region.y,
            self.region.width as u32,
            self.region.height as u32,
        )?;

        let (width, height) = screen.dimensions();
        let mut mat = Mat::new_rows_cols_with_default(
            height as i32,
            width as i32,
            CV_8UC4,
            Scalar::all(0.0),
        )?;

        let mat_data = mat.data_bytes_mut()?;
        mat_data.copy_from_slice(&screen.as_raw());

        let mut mat_bgr = Mat::default();
        imgproc::cvt_color(&mat, &mut mat_bgr, imgproc::COLOR_RGBA2BGR, 0)?;

        Ok(mat_bgr)
    }

    pub fn load_pieces(&self) -> Result<HashMap<String, (Mat, f64)>, Box<dyn std::error::Error>> {
        let mut pieces: HashMap<String, (Mat, f64)> = HashMap::new();

        for entry in fs::read_dir("../chesscom/").unwrap() {
            if let Ok(entry) = entry {
                // let file_name = entry.file_name().into_string().unwrap();
                let file_name = entry
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let img = imgcodecs::imread(
                    &entry.path().to_str().unwrap(),
                    imgcodecs::IMREAD_UNCHANGED,
                )?;
                pieces.insert(
                    file_name.clone(),
                    (img, *self.thresholds.get(&file_name).unwrap()),
                );
            }
        }

        Ok(pieces)
    }

    pub fn process_image(
        &self,
        board_image: &Mat,
        pieces: HashMap<String, (Mat, f64)>,
    ) -> Result<[[char; 8]; 8], Box<dyn std::error::Error>> {
        let mut result: [[char; 8]; 8] = [[' '; 8]; 8];
        let mut board_clone = board_image.clone();

        for piece_name in pieces.keys() {
            let piece_threshold = pieces.get(piece_name).unwrap().clone().1;
            let mut piece_image = pieces.get(piece_name).unwrap().clone().0;
            // WA for chesscom
            if *piece_name == "p".to_string() {
                piece_image = utils::resize(&piece_image, 43, 43).unwrap();
            }

            let mut board_gray = Mat::default();
            imgproc::cvt_color(board_image, &mut board_gray, imgproc::COLOR_BGR2GRAY, 0).unwrap();

            let mut piece_gray = Mat::default();
            imgproc::cvt_color(&piece_image, &mut piece_gray, imgproc::COLOR_BGR2GRAY, 0).unwrap();

            // let mut mask = Mat::default();
            let mut channels: Vector<Mat> = Vector::new();
            // if piece_image.channels() == 4 {
            split(&piece_image, &mut channels).unwrap();
            let mask = channels.get(3).unwrap();
            // }
            let size = piece_gray.size().unwrap();
            let (h, w) = (size.height, size.width);

            let mut matched = Mat::default();
            imgproc::match_template(
                &board_gray,
                &piece_gray,
                &mut matched,
                imgproc::TM_SQDIFF_NORMED,
                &mask,
            )?;

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
                    Scalar::new(255.0, 0.0, 0.0, 0.0) // Red for black pieces
                } else {
                    Scalar::new(0.0, 0.0, 255.0, 0.0) // Blue for white pieces
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

                // highgui::imshow("abc", &board_clone);
                // loop {
                //     if highgui::wait_key(0)? == 48 {
                //         break;
                //     }
                // }
                // let params = Vector::from_iter([0, 16]);
                // imgcodecs::imwrite("rust_result.jpg", &board_clone, &params);

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
                // println!("NEXT {min_val} {max_val} {:?} {:?}", min_loc, max_loc);
            }
        }
        // let params = Vector::from_iter([0, 16]);
        // imgcodecs::imwrite("rust_result.jpg", &board_clone, &params);
        Ok(result)
    }
}

fn main() {
    let total = std::time::Instant::now();
    let st = std::time::Instant::now();
    let tracker = ChessboardTracker::new(Rect::new(440, 219, 758, 759));
    println!("tracker: {:?}", st.elapsed());

    let st = std::time::Instant::now();
    let image = tracker.capture_screenshot().unwrap();
    println!("screenshot: {:?}", st.elapsed());

    let st = std::time::Instant::now();
    let resized = utils::resize(&image, 360, 360).unwrap();
    println!("resize: {:?}", st.elapsed());

    let st = std::time::Instant::now();
    let pieces = tracker.load_pieces().unwrap();
    println!("laod pieces: {:?}", st.elapsed());

    let st = std::time::Instant::now();
    let result = tracker.process_image(&resized, pieces);
    println!("process: {:?}", st.elapsed());
    println!("TOTOAL: {:?}", total.elapsed());
}
