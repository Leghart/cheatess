mod config;
mod engine;
mod image;
mod stockfish;
mod utils;

pub mod webwrapper;
use image::ImageProcessing;
use opencv::imgcodecs;
use std::time::Instant;
use webwrapper::chesscom::ChesscomWrapper;
use webwrapper::ChessboardTrackerInterface;

fn main() {
    // _single_run();
    _loop();
}

fn _single_run() {
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
    let _ = tracker.process_image(&resized, &pieces).unwrap();

    println!("process: {:?}", st.elapsed());
    println!("TOTOAL: {:?}", total.elapsed());

    println!("\x1b[33m♕\x1b[0m");
}

fn _loop() {
    let tracker_def = ChesscomWrapper::default();
    // 440 219 758 759
    // 445, 185, 742, 743
    let tracker = ChesscomWrapper::new(
        utils::screen_region::ScreenRegion::new(443, 183, 744, 745),
        tracker_def.get_thresholds().clone(),
    );
    println!("tracker region: {:?}", tracker.get_region());
    let pieces = tracker.load_pieces().unwrap();

    loop {
        let st = std::time::Instant::now();
        // println!("start: {:?}", st.elapsed());

        let image = tracker.capture_screenshot().unwrap();
        // println!("screenshot: {:?}", st.elapsed());

        let resized = ImageProcessing::resize(&image, 360, 360).unwrap();
        // println!("resize: {:?}", st.elapsed());

        let board_data = tracker.process_image(&resized, &pieces).unwrap();
        // println!("process: {:?}", st.elapsed());

        let board = engine::Board::new(board_data);
        board.print();
        // println!("{:?}", file_name);
        // println!("cycle: {:?}", st.elapsed());
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
