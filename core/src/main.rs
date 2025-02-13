use opencv::{
    core::{KeyPoint, Mat, Point, Rect, Scalar, Size, Vector},
    features2d::{BFMatcher, ORB_ScoreType, ORB},
    highgui, imgcodecs, imgproc,
    prelude::*,
};
use screenshots::Screen;
use std::{collections::HashMap, thread, time::Duration};

pub struct ChessboardTracker {
    region: Rect,
    pieces_images: HashMap<char, Mat>,
}

impl ChessboardTracker {
    pub fn new(region: Rect) -> Self {
        ChessboardTracker {
            region,
            pieces_images: HashMap::new(),
        }
    }

    pub fn capture_screenshot(&self) -> Result<Mat, Box<dyn std::error::Error>> {
        let screen = Screen::all()?.first().unwrap().capture_area(
            self.region.x,
            self.region.y,
            self.region.width as u32,
            self.region.height as u32,
        )?;
        let image = Mat::from_slice(screen.as_raw())?;
        Ok(image.try_clone()?)
    }

    pub fn process_image(&self, image: &Mat) -> Result<[[char; 8]; 8], Box<dyn std::error::Error>> {
        let mut board: [[char; 8]; 8] = [[' '; 8]; 8];

        // Konwersja do odcieni szarości
        let mut gray = Mat::default();
        imgproc::cvt_color(image, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

        // ORB Feature Detector
        let mut orb = ORB::create(500, 1.2, 8, 31, 0, 2, ORB_ScoreType::FAST_SCORE, 31, 20)?;
        let mut keypoints_board = Vector::new();
        let mut descriptors_board = Mat::default();
        orb.detect_and_compute(
            &gray,
            &Mat::default(),
            &mut keypoints_board,
            &mut descriptors_board,
            false,
        )?;

        let mut matcher = BFMatcher::create(6, true)?;

        for (piece, template) in &self.pieces_images {
            let mut keypoints_piece = Vector::new();
            let mut descriptors_piece = Mat::default();
            orb.detect_and_compute(
                template,
                &Mat::default(),
                &mut keypoints_piece,
                &mut descriptors_piece,
                false,
            )?;

            let mut matches = Vector::new();
            matcher.match_(&descriptors_piece, &mut matches, &Mat::default())?;

            if matches.len() > 10 {
                let avg_x = matches
                    .iter()
                    .map(|m| keypoints_board.get(m.train_idx as usize).unwrap().pt().x as usize)
                    .sum::<usize>()
                    / matches.len();
                let avg_y = matches
                    .iter()
                    .map(|m| keypoints_board.get(m.train_idx as usize).unwrap().pt().y as usize)
                    .sum::<usize>()
                    / matches.len();

                let row = (avg_y / (self.region.height / 8) as usize) as usize;
                let col = (avg_x / (self.region.width / 8) as usize) as usize;
                board[row][col] = *piece;
            }
        }

        Ok(board)
    }
}

fn main() {
    let tracker = ChessboardTracker::new(Rect::new(100, 100, 400, 400));
    let image = imgcodecs::imread("boards/ccc.png", imgcodecs::IMREAD_COLOR).unwrap();
    match tracker.process_image(&image) {
        Ok(board) => println!("{:?}", board),
        Err(e) => eprintln!("Error processing image: {}", e),
    }

    // loop {
    //     if let Ok(image) = tracker.capture_screenshot() {
    //         match tracker.process_image(&image) {
    //             Ok(board) => println!("{:?}", board),
    //             Err(e) => eprintln!("Error processing image: {}", e),
    //         }
    //     }
    //     thread::sleep(Duration::from_millis(100));
    // }
}
