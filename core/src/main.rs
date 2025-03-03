mod config;
mod engine;
mod game;
mod image;
mod stockfish;
mod utils;
pub mod webwrapper;

use crate::webwrapper::ChessboardTrackerInterface;
use config::save_config;
use image::ImageProcessing;
use stockfish::Stockfish;
use utils::file_system::RealFileSystem;
use webwrapper::chesscom::ChesscomWrapper;

fn main() {}

fn run() {
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
    let _ = tracker.process_image(&resized, pieces).unwrap();

    println!("process: {:?}", st.elapsed());
    println!("TOTOAL: {:?}", total.elapsed());
}

fn store_cfg() {
    let conf = config::Config::new(
        webwrapper::WrapperMode::Chesscom,
        utils::screen_region::ScreenRegion::new(70, 70, 700, 700),
        std::collections::HashMap::from_iter([('C', 0.6721)]),
        false,
        String::new(),
    )
    .unwrap();
    save_config(&conf, &mut RealFileSystem).unwrap();
}
