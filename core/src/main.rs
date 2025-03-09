// mod config;
// mod engine;
// mod image;
// mod stockfish;
// mod utils;

// pub mod webwrapper;

// use crate::webwrapper::ChessboardTrackerInterface;
// use config::save_config;
// use image::ImageProcessing;

// use utils::{
//     context::{Context, MsgKey, ProtocolInterface},
//     file_system::RealFileSystem,
//     screen_region::ScreenRegion,
// };
// use webwrapper::chesscom::ChesscomWrapper;

// fn main() {
//     let mut ctx = Context::new();
//     ctx.connect();

//     let mut region = ScreenRegion::new(0, 0, 0, 0);

//     loop {
//         let msg = ctx.recv();

//         match msg.key {
//             MsgKey::Region => {
//                 match ScreenRegion::try_from(msg.message) {
//                     Ok(_region) => region = _region,
//                     Err(err) => eprintln!("{err}"),
//                 };
//                 ctx.send(ProtocolInterface {
//                     key: MsgKey::Ok,
//                     message: String::new(),
//                 });
//             }
//             MsgKey::Configurate => {}
//             MsgKey::Ping => {
//                 let response = ProtocolInterface {
//                     key: MsgKey::Ok,
//                     message: format!("{:?}", region),
//                 };
//                 ctx.send(response);
//             }
//             MsgKey::Game => {
//                 // will block main thread
//             }
//             MsgKey::Ok => {}
//         }
//     }
// }

// fn _save_config() {
//     let conf = config::Config::new(
//         webwrapper::WrapperMode::Chesscom,
//         utils::screen_region::ScreenRegion::new(70, 70, 700, 700),
//         std::collections::HashMap::from_iter([('C', 0.6721)]),
//         false,
//         String::new(),
//     )
//     .unwrap();
//     save_config(&conf, &mut RealFileSystem).unwrap();
// }

// fn _single_run() {
//     let total = std::time::Instant::now();
//     let st = std::time::Instant::now();
//     let tracker = ChesscomWrapper::default();
//     println!("tracker: {:?}", st.elapsed());

//     let st = std::time::Instant::now();
//     let image = tracker.capture_screenshot().unwrap();
//     println!("screenshot: {:?}", st.elapsed());

//     let st = std::time::Instant::now();
//     let resized = ImageProcessing::resize(&image, 360, 360).unwrap();
//     println!("resize: {:?}", st.elapsed());

//     let st = std::time::Instant::now();
//     let pieces = tracker.load_pieces().unwrap();
//     println!("laod pieces: {:?}", st.elapsed());

//     let st = std::time::Instant::now();
//     let _ = tracker.process_image(&resized, pieces).unwrap();

//     println!("process: {:?}", st.elapsed());
//     println!("TOTOAL: {:?}", total.elapsed());
// }
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::{handle, AggregatedMessage, Message, Session};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Command {
    action: String,
    parameters: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Config {
    setting1: String,
    setting2: i32,
}

struct AppState {
    commands: Mutex<Vec<Command>>,
    config: Mutex<Config>,
}

// WebSocket for game communication
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    actix_web::rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    if let Ok(command) = serde_json::from_str::<Command>(&text) {
                        println!("Received command: {:?}", command);
                        let json_message = serde_json::to_string(&command).unwrap();
                        // session.text(ByteString::from(json_message)).await.unwrap();
                    } else {
                        println!("Invalid command received: {}", text);
                    }
                }
                Ok(AggregatedMessage::Binary(bin)) => {
                    session.binary(bin).await.unwrap();
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }
                _ => {}
            }
        }
    });

    Ok(res)
}

// HTTP GET to fetch current configuration
#[get("/config")]
async fn get_config(data: web::Data<AppState>) -> impl Responder {
    let config = data.config.lock().unwrap();
    HttpResponse::Ok().json(&*config)
}

// HTTP POST to update configuration
#[post("/config")]
async fn set_config(data: web::Data<AppState>, new_config: web::Json<Config>) -> impl Responder {
    let mut config = data.config.lock().unwrap();
    *config = new_config.into_inner();
    HttpResponse::Ok().json("Configuration updated")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(AppState {
        commands: Mutex::new(vec![]),
        config: Mutex::new(Config {
            setting1: "default".to_string(),
            setting2: 42,
        }),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .route("/ws/game", web::get().to(websocket_handler))
            .service(get_config)
            .service(set_config)
    })
    .bind("127.0.0.1:5555")?
    .run()
    .await
}
