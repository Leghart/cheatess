use image::imageops;

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

    match args.mode {
        parser::Mode::Game => game(args),
        parser::Mode::Test => config_mode(args).expect("Test config failed"),
    }
}

fn game(args: parser::CheatessArgs) {
    clear_screen();

    let mut stdout = io::stdout();
    let mut sf = stockfish::Stockfish::new(&args.stockfish.path, args.stockfish.depth);
    sf.set_config(
        &args.stockfish.elo.to_string(),
        &args.stockfish.skill.to_string(),
        &args.stockfish.hash.to_string(),
    );

    let monitor =
        monitor::select_monitor(args.monitor.number).expect("Requested monitor not found");
    let raw = monitor::capture_entire_screen(&monitor);
    let entire_screen_gray = procimg::image_buffer_to_gray_mat(&raw).unwrap();
    let coords = procimg::get_board_region(&entire_screen_gray);

    let cropped = imageops::crop_imm(&raw, coords.0, coords.1, coords.2, coords.3).to_image();
    let board = procimg::image_buffer_to_gray_mat(&cropped).unwrap();

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
    let best_move = sf.get_best_move().unwrap();
    log::info!("Stockfish best move: {best_move}");
    log::info!("Evaluation: {:?}", sf.get_evaluation());

    loop {
        let start = Instant::now();
        let cropped = monitor::get_cropped_screen(&monitor, coords.0, coords.1, coords.2, coords.3); // ~25ms
        let gray_board = procimg::image_buffer_to_gray_mat(&cropped).unwrap(); // ~20ms
        log::trace!("image preparation: {:?}", start.elapsed());

        if !procimg::are_images_different(&prev_board_mat, &gray_board, 500) {
            continue;
        }

        let new_raw_board = procimg::find_all_pieces(
            &gray_board,
            &pieces,
            args.proc_image.piece_threshold,
            args.proc_image.board_threshold,
        );
        log::trace!("Pieces detection: {:?}", start.elapsed());

        let detected_move =
            engine::detect_move(prev_board_arr.raw(), &new_raw_board, &player_color);

        match detected_move {
            Ok((mv, mv_type)) => {
                log::info!("Detected move: {mv:?} [{mv_type:?}]");
                sf.make_move(vec![mv]);
            }
            Err(e) => {
                log::error!("{e}");
                continue;
            }
        }
        clear_screen();

        let curr_board: Box<dyn engine::AnyBoard> = if args.engine.pretty_pieces {
            engine::create_board_from_data::<engine::PrettyPrinter>(new_raw_board, &player_color)
        } else {
            engine::create_board_from_data::<engine::DefaultPrinter>(new_raw_board, &player_color)
        };
        curr_board.print(&mut stdout);

        match sf.get_best_move() {
            Some(best) => {
                log::info!("Stockfish best move: {best}");
                log::info!("Evaluation: {:?}", sf.get_evaluation());
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

fn config_mode(args: parser::CheatessArgs) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("{:?}", args.monitor);
    log::info!("{:?}", args.engine);
    log::info!("{:?}", args.proc_image);
    log::info!("{:?}", args.stockfish);

    log::info!("\nNow you will see the following images: entire screen in grayscale, cropped board from previus image,...");
    log::info!("To get next image, press '0'");

    let monitor =
        monitor::select_monitor(args.monitor.number).expect("Requested monitor not found");
    let raw = monitor::capture_entire_screen(&monitor);
    let entire_screen_gray = procimg::image_buffer_to_gray_mat(&raw).unwrap();
    procimg::show(&entire_screen_gray, true, "Entire screen")?;

    let coords = procimg::get_board_region(&entire_screen_gray);
    let cropped = imageops::crop_imm(&raw, coords.0, coords.1, coords.2, coords.3).to_image();
    let board = procimg::image_buffer_to_gray_mat(&cropped).unwrap();
    procimg::show(&board, true, "Cropped board")?;

    let player_color = procimg::detect_player_color(&board);
    log::warn!("\nDetected player color: {player_color:?}");

    log::info!("\nNow you will see all extracted pieces from board, please check if everyone is clear and has high resolution");
    log::info!("If image is bad, you can improve it by change proc_image arguments: margin and extract_piece_threshold");
    let pieces = procimg::extract_pieces(
        &board,
        args.proc_image.margin,
        args.proc_image.extract_piece_threshold,
        &player_color,
    )?;

    for (sign, mat) in &pieces {
        procimg::show(&mat, true, &format!("Extracted piece: {sign}"))?;
    }

    log::info!("Binary board");
    let bin_board = procimg::convert_board_to_bin(&board, args.proc_image.board_threshold);
    procimg::show(&bin_board, true, "Binary board")?;

    log::info!("Last step, check if every piece is correctly placed");
    let pieces = pieces
        .into_iter()
        .map(|(c, mat)| (c, Arc::new(mat)))
        .collect();

    let raw_board = procimg::find_all_pieces(
        &board,
        &pieces,
        args.proc_image.piece_threshold,
        args.proc_image.board_threshold,
    );

    let board: Box<dyn engine::AnyBoard> = if args.engine.pretty_pieces {
        engine::create_board_from_data::<engine::PrettyPrinter>(raw_board, &player_color)
    } else {
        engine::create_board_from_data::<engine::DefaultPrinter>(raw_board, &player_color)
    };
    board.print(&mut io::stdout());

    Ok(())
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}
