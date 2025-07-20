use image::{imageops, DynamicImage};

use logger::Logger;
use std::io;
use std::sync::Arc;
use std::time::Instant;

mod engine;
mod logger;
mod monitor;
mod parser;
mod printer;
mod procimg;
mod stockfish;

static LOGGER: Logger = Logger;

fn main() {
    run();
}

fn run() {
    let env_args: Vec<String> = std::env::args().collect();
    let args = parser::parse_args_from(env_args);

    logger::init(&args.verbose.log_level_filter());

    clear_screen();

    let mut stdout = io::stdout();
    let mut st = stockfish::Stockfish::new(&args.stockfish.path, args.stockfish.depth);
    st.set_config(
        &args.stockfish.elo.to_string(),
        &args.stockfish.skill.to_string(),
        &args.stockfish.hash.to_string(),
    );

    let monitor =
        monitor::select_monitor(args.monitor.number).expect("Requested monitor not found");
    let raw = monitor::capture_entire_screen(&monitor);
    let dyn_image = DynamicImage::ImageRgba8(raw.clone());
    let entire_screen_gray = procimg::dynamic_image_to_gray_mat(&dyn_image).unwrap();

    let coords = procimg::get_board_region(&entire_screen_gray);

    let cropped = imageops::crop_imm(&raw, coords.0, coords.1, coords.2, coords.3).to_image();
    let dyn_image = DynamicImage::ImageRgba8(cropped);
    let board = procimg::dynamic_image_to_gray_mat(&dyn_image).unwrap();

    let player_color = procimg::detect_player_color(&board);
    log::info!("Detected player color: {player_color:?}");

    let base_board: Box<dyn engine::AnyBoard> = if args.engine.pretty_pieces {
        engine::create_board_default::<engine::PrettyPrinter>(&player_color)
    } else {
        engine::create_board_default::<engine::DefaultPrinter>(&player_color)
    };
    base_board.print(&mut stdout);

    let pieces = procimg::extract_pieces(
        &board,
        args.proc_image.margin,
        args.proc_image.extract_piece_threshold,
        &player_color,
    )
    .unwrap();
    let pieces = pieces
        .into_iter()
        .map(|(c, mat)| (c, Arc::new(mat)))
        .collect();

    let mut prev_board_mat = board;
    let mut prev_board_arr = base_board;
    let best_move = st.get_best_move().unwrap();
    log::info!("Stockfish best move: {best_move}");
    log::info!("Evaluation: {:?}", st.get_evaluation());

    loop {
        let start = Instant::now();
        let cropped = monitor::get_cropped_screen(&monitor, coords.0, coords.1, coords.2, coords.3); // ~25ms
        let dyn_image = DynamicImage::ImageRgba8(cropped); // ~25ms
        let gray_board = procimg::dynamic_image_to_gray_mat(&dyn_image).unwrap(); // ~20ms

        if !procimg::are_images_different(&prev_board_mat, &gray_board, 500) {
            continue;
        }

        let new_raw_board =
            procimg::find_all_pieces(&gray_board, &pieces, args.proc_image.piece_threshold);
        log::trace!("Pieces detection: {:?}", start.elapsed());

        let detected_move =
            engine::detect_move(prev_board_arr.raw(), &new_raw_board, &player_color);

        clear_screen();
        match detected_move {
            Ok((mv, mv_type)) => {
                log::info!("Detected move: {mv:?} [{mv_type:?}]");
                st.make_move(vec![mv]);
            }
            Err(e) => {
                log::error!("{e}");
            }
        }

        let curr_board: Box<dyn engine::AnyBoard> = if args.engine.pretty_pieces {
            engine::create_board_from_data::<engine::PrettyPrinter>(new_raw_board, &player_color)
        } else {
            engine::create_board_from_data::<engine::DefaultPrinter>(new_raw_board, &player_color)
        };

        curr_board.print(&mut stdout);
        match st.get_best_move() {
            Some(best) => {
                log::info!("Stockfish best move: {best}");
                log::info!("Evaluation: {:?}", st.get_evaluation());
            }
            None => {
                log::info!("Game over");
                break;
            }
        }

        prev_board_arr = curr_board;
        prev_board_mat = gray_board;
        log::debug!("Cycle time: {:?}", start.elapsed());
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}
