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

    let base_board: Box<dyn engine::AnyBoard> = if args.engine.pretty {
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

        if !procimg::are_images_different(
            &prev_board_mat,
            &gray_board,
            args.proc_image.difference_level,
        ) {
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

        let curr_board: Box<dyn engine::AnyBoard> = if args.engine.pretty {
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
    log::info!("Welcome to the interactive test setup for cheatess. Follow the instructions to ensure everything works correctly while playing.");

    log::info!("\n[Step 1/7] Collected invoke parameters:");
    log::info!("{:?}", args.monitor);
    log::info!("{:?}", args.engine);
    log::info!("{:?}", args.proc_image);
    log::info!("{:?}", args.stockfish);

    log::info!("\n[Step 2/7] Now you will see the following images: entire screen in grayscale and cropped board from previus image");
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
    log::warn!("\n[Step 3/7] Detected player color: {player_color:?}");

    log::info!("\n[Step 4/7] Now you will see all extracted pieces from board, please check if every is clear");
    log::info!("If image is bad, you can improve it by change imgproc arguments: margin (-m) and extract_piece_threshold (-e)");
    let pieces = procimg::extract_pieces(
        &board,
        args.proc_image.margin,
        args.proc_image.extract_piece_threshold,
        &player_color,
    )?;

    for (sign, mat) in &pieces {
        procimg::show(mat, true, &format!("Extracted piece: {sign}"))?;
    }

    log::info!("[Step 5/7] Now you will see board converted to binary...");
    let bin_board = procimg::convert_board_to_bin(&board, args.proc_image.board_threshold);
    procimg::show(&bin_board, true, "Binary board")?;

    log::info!("[Step 6/7] Now check if every piece is correctly placed");
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

    let calc_board: Box<dyn engine::AnyBoard> = if args.engine.pretty {
        engine::create_board_from_data::<engine::PrettyPrinter>(raw_board, &player_color)
    } else {
        engine::create_board_from_data::<engine::DefaultPrinter>(raw_board, &player_color)
    };
    calc_board.print(&mut io::stdout());

    log::info!("[Step 7/7] Last step, make any move on your web board and check if move detection is correct (you can configure it with -d flag)");
    log::info!("If you make an exactly one move, press enter...");
    io::stdin().read_line(&mut String::new())?;

    let prev_board = board;
    let prev_board_arr = calc_board;
    let new_cropped = monitor::get_cropped_screen(&monitor, coords.0, coords.1, coords.2, coords.3);
    let new_board = procimg::image_buffer_to_gray_mat(&new_cropped).unwrap();

    if !procimg::are_images_different(&prev_board, &new_board, args.proc_image.difference_level) {
        log::error!("Not detected the move");
        return Err("Not detected the move".into());
    }

    let new_raw_board = procimg::find_all_pieces(
        &new_board,
        &pieces,
        args.proc_image.piece_threshold,
        args.proc_image.board_threshold,
    );

    let (detected_move, _) =
        engine::detect_move(prev_board_arr.raw(), &new_raw_board, &player_color)
            .expect("Not found a move");

    log::info!("Detected move: {detected_move}");

    Ok(())
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}
