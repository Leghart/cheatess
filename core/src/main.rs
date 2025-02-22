mod engine;
mod game;
mod image;
pub mod webwrapper;

use crate::webwrapper::ChessboardTrackerInterface;
use image::ImageProcessing;
use webwrapper::chesscom::ChesscomWrapper;

fn main() {
    let total = std::time::Instant::now();
    let st = std::time::Instant::now();
    let tracker = ChesscomWrapper::default();
    println!("tracker: {:?}", st.elapsed());

    let st = std::time::Instant::now();
    let image = tracker.capture_screenshot().unwrap();
    println!("screenshot: {:?}", st.elapsed());

    let st = std::time::Instant::now();
    let resized = ImageProcessing::resize(&image, 360, 360).unwrap();
    println!("resize: {:?}", st.elapsed());

    let st = std::time::Instant::now();
    let pieces = tracker.load_pieces().unwrap();
    println!("laod pieces: {:?}", st.elapsed());

    let st = std::time::Instant::now();
    let result = tracker.process_image(&resized, pieces).unwrap();
    // println!("{:?}", result);
    for i in result.iter().rev() {
        println!("{:?}", i);
    }
    println!("process: {:?}", st.elapsed());
    println!("TOTOAL: {:?}", total.elapsed());
}
