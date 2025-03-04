mod config;
mod engine;
mod image;
mod stockfish;
mod utils;
pub mod webwrapper;
use zmq::Socket;

use crate::webwrapper::ChessboardTrackerInterface;
use config::save_config;
use image::ImageProcessing;
use serde::{Deserialize, Serialize};

use utils::file_system::RealFileSystem;
use webwrapper::chesscom::ChesscomWrapper;

#[derive(Serialize, Deserialize, Debug)]
enum MsgKey {
    Configurate,
    Ping,
    Game,
    Region,
    Ok,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProtocolInterface {
    key: MsgKey,
    message: String,
}

fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REP).expect("Fatal error");
    socket.bind("tcp://127.0.0.1:5555").expect("Fatal error");

    loop {
        let msg = recv(&socket);

        match msg.key {
            MsgKey::Region => {
                println!("AAA: {:?}", msg.message);
                send(
                    &socket,
                    ProtocolInterface {
                        key: MsgKey::Ok,
                        message: String::new(),
                    },
                );
            }
            MsgKey::Configurate => {}
            MsgKey::Ping => {
                let response = ProtocolInterface {
                    key: MsgKey::Ping,
                    message: format!("ping"),
                };
                send(&socket, response);
            }
            MsgKey::Game => {
                // will block main thread
            }
            MsgKey::Ok => {}
        }
    }
}

fn recv(socket: &Socket) -> ProtocolInterface {
    let msg = socket.recv_string(0).expect("none").unwrap();
    serde_json::from_str(&msg).unwrap()
}

fn send(socket: &Socket, msg: ProtocolInterface) {
    let response_str = serde_json::to_string(&msg).unwrap();
    socket.send(&response_str, 0).expect("sendind error");
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
