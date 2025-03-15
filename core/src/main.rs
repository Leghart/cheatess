mod config;
mod engine;
mod image;
mod stockfish;
mod utils;
pub mod webwrapper;

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
//     let conf = config::GameConfig::new(
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
use actix_cors::Cors;
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::{handle, AggregatedMessage};
use config::stockfish::StockfishConfig;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Command {
    action: String,
    parameters: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GameConfig {
    platform: String,
    theme: String,
    thresholds: Vec<f64>,
}

struct AppState {
    commands: Mutex<Vec<Command>>,
    game_config: Mutex<GameConfig>,
    stockfish_config: Mutex<StockfishConfig>,
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
                        let json_message = serde_json::to_string(&command).unwrap();
                        session.text(json_message).await.unwrap();
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

#[get("/config")]
async fn get_config(data: web::Data<AppState>) -> impl Responder {
    let config = data.game_config.lock().unwrap();
    HttpResponse::Ok().json(&*config)
}

#[post("/config")]
async fn set_config(
    data: web::Data<AppState>,
    new_config: web::Json<GameConfig>,
) -> impl Responder {
    let mut config = data.game_config.lock().unwrap();
    *config = new_config.into_inner();
    HttpResponse::Ok().json("Configuration updated")
}

#[get("/stockfish/config")]
async fn get_stock_config(data: web::Data<AppState>) -> impl Responder {
    let config = data.stockfish_config.lock().unwrap();
    HttpResponse::Ok().json(&*config)
}

#[post("/stockfish/config")]
async fn set_stock_config(
    data: web::Data<AppState>,
    new_config: web::Json<StockfishConfig>,
) -> impl Responder {
    let mut config = data.stockfish_config.lock().unwrap();
    *config = new_config.into_inner();
    HttpResponse::Ok().json("Configuration updated")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let stockfish_config =
        config::stockfish::StockfishConfig::new("abc", None, None, None).unwrap();

    let app_data = web::Data::new(AppState {
        commands: Mutex::new(vec![]),
        game_config: Mutex::new(GameConfig {
            theme: "default".to_string(),
            thresholds: Vec::new(),
            platform: "".to_string(),
        }),
        stockfish_config: Mutex::new(stockfish_config),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:1420")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec!["Content-Type"])
                    .max_age(3600),
            )
            .app_data(app_data.clone())
            .route("/ws/game", web::get().to(websocket_handler))
            .service(get_config)
            .service(set_config)
            .service(get_stock_config)
            .service(set_stock_config)
    })
    .bind("127.0.0.1:5555")?
    .run()
    .await
}
