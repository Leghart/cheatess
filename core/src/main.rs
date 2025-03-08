mod config;
mod engine;
mod image;
mod stockfish;
mod utils;

pub mod webwrapper;

use crate::webwrapper::ChessboardTrackerInterface;
use config::save_config;
use image::ImageProcessing;

use utils::{
    context::{Context, MsgKey, ProtocolInterface},
    file_system::RealFileSystem,
    screen_region::ScreenRegion,
};
use webwrapper::chesscom::ChesscomWrapper;

fn main() {
    let mut ctx = Context::new();
    ctx.connect();

    let mut region = ScreenRegion::new(0, 0, 0, 0);

    loop {
        let msg = ctx.recv();

        match msg.key {
            MsgKey::Region => {
                match ScreenRegion::try_from(msg.message) {
                    Ok(_region) => region = _region,
                    Err(err) => eprintln!("{err}"),
                };
                ctx.send(ProtocolInterface {
                    key: MsgKey::Ok,
                    message: String::new(),
                });
            }
            MsgKey::Configurate => {}
            MsgKey::Ping => {
                let response = ProtocolInterface {
                    key: MsgKey::Ok,
                    message: format!("{:?}", region),
                };
                ctx.send(response);
            }
            MsgKey::Game => {
                // will block main thread
            }
            MsgKey::Ok => {}
        }
    }
}

fn _save_config() {
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
    let _ = tracker.process_image(&resized, pieces).unwrap();

    println!("process: {:?}", st.elapsed());
    println!("TOTOAL: {:?}", total.elapsed());
}
