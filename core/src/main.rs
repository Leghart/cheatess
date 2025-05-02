mod config;
mod engine;
mod image;
mod stockfish;
mod utils;

pub mod webwrapper;
use image::ImageProcessing;
use webwrapper::chesscom::ChesscomWrapper;
use webwrapper::ChessboardTrackerInterface;
fn main() {
    _single_run();
}

fn _single_run() {
    // let total = std::time::Instant::now();
    // let st = std::time::Instant::now();
    // let tracker = ChesscomWrapper::default();
    // println!("tracker: {:?}", st.elapsed());

    // let st = std::time::Instant::now();
    // let image = tracker.capture_screenshot().unwrap();
    // println!("screenshot: {:?}", st.elapsed());

    // let st = std::time::Instant::now();
    // let resized = ImageProcessing::resize(&image, 360, 360).unwrap();
    // println!("resize: {:?}", st.elapsed());

    // let st = std::time::Instant::now();
    // let pieces = tracker.load_pieces().unwrap();
    // println!("laod pieces: {:?}", st.elapsed());

    // let st = std::time::Instant::now();
    // let _ = tracker.process_image(&resized, pieces).unwrap();

    // println!("process: {:?}", st.elapsed());
    // println!("TOTOAL: {:?}", total.elapsed());
    let b = engine::Board::new();
    b.print();

    // println!("♔♕♖♗♘♙♟♞♝♜♛♚");

    println!("\x1b[33m♕\x1b[0m");
}
