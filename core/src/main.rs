use image::{imageops, DynamicImage};
use std::io;
use std::sync::Arc;
use std::time::Instant;

mod engine;
mod monitor;
mod procimg;
mod stockfish;

fn main() {
    run();
}

fn run() {
    clear_screen();
    let mut stdout = io::stdout();
    let mut st =
        stockfish::Stockfish::new("/home/leghart/projects/cheatess/stockfish-ubuntu-x86-64-avx2");
    st.set_config();
    st.set_elo_rating(2000);

    let monitor = monitor::select_monitor(true).expect("No primary monitor found");
    let raw = monitor::capture_entire_screen(&monitor);
    let dyn_image = DynamicImage::ImageRgba8(raw.clone());
    let entire_screen_gray = procimg::dynamic_image_to_gray_mat(&dyn_image).unwrap();

    let coords = procimg::get_board_region(&entire_screen_gray);

    let cropped = imageops::crop_imm(&raw, coords.0, coords.1, coords.2, coords.3).to_image();
    let dyn_image = DynamicImage::ImageRgba8(cropped);
    let board = procimg::dynamic_image_to_gray_mat(&dyn_image).unwrap();

    let player_color = procimg::detect_player_color(&board);

    let base_board = engine::create_board::<engine::PrettyPrinter>(&player_color);
    base_board.print(&mut stdout);

    let pieces = procimg::extract_pieces(&board, &player_color).unwrap();
    let pieces = pieces
        .into_iter()
        .map(|(c, mat)| (c, Arc::new(mat)))
        .collect();

    let mut prev_board_mat = board;
    let mut prev_board_arr = base_board;
    let best_move = st.get_best_move().unwrap();
    println!("Stockfish best move: {}", best_move);

    println!("---->{:?}", st.get_evaluation());

    loop {
        let start = Instant::now();
        let cropped = monitor::get_cropped_screen(&monitor, coords.0, coords.1, coords.2, coords.3);
        let dyn_image = DynamicImage::ImageRgba8(cropped);
        let gray_board = procimg::dynamic_image_to_gray_mat(&dyn_image).unwrap();

        if !procimg::images_have_differences(&prev_board_mat, &gray_board, 500) {
            continue;
        }

        let new_raw_board = procimg::find_all_pieces(&gray_board, &pieces);

        let detected_move = engine::detect_move(&prev_board_arr.raw, &new_raw_board, &player_color);

        if let Some(mv) = detected_move {
            println!("Detected move: {:?}", mv);
            st.make_move(vec![mv]);
        } else {
            println!("not found move");
        }

        clear_screen();
        let curr_board = engine::Board::new(new_raw_board);
        curr_board.print(&mut stdout);
        let best_move = st.get_best_move().unwrap();
        println!("Stockfish best move: {}", best_move);
        println!("---->{:?}", st.get_evaluation());
        println!("====>{:?}", st.get_wdl_stats());

        prev_board_arr = curr_board;
        prev_board_mat = gray_board;
        println!("Time taken: {:?}", start.elapsed());
    }
}

/// Clears the terminal screen by printing escape sequences.
fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}
