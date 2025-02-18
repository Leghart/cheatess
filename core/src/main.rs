mod image;
pub mod utils;
pub mod webwrapper;

use crate::webwrapper::ChessboardTrackerInterface;
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
    let resized = utils::resize(&image, 360, 360).unwrap();
    println!("resize: {:?}", st.elapsed());
    // let params = Vector::from_iter([0, 16]);
    // imgcodecs::imwrite("aaa.jpg", &resized, &params);

    let st = std::time::Instant::now();
    let pieces = tracker.load_pieces().unwrap();
    println!("laod pieces: {:?}", st.elapsed());

    let st = std::time::Instant::now();
    let result = tracker.process_image(&resized, pieces);
    println!("process: {:?}", st.elapsed());
    println!("TOTOAL: {:?}", total.elapsed());
}
