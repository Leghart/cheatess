extern crate opencv;

mod engine;
mod image;
use image::ImageProcessing;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let st = Instant::now();
    let mut proc = ImageProcessing::default();
    proc.load_piece_images("chess_piece")?; //TODO: move to constructor
    let img = proc.load_image("test/board.jpg")?;

    let board = proc.image_to_board(&img);

    for row in board.iter() {
        println!("{:?}", row);
    }

    // let after = [
    //     ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
    //     ['P', 'P', 'P', ' ', 'P', 'P', 'P', 'P'],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', 'P', 'p', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     ['p', 'p', 'p', 'p', ' ', 'p', 'p', 'p'],
    //     ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
    // ];

    // let before = [
    //     ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
    //     ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', 'P', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     ['p', 'p', 'p', 'p', ' ', 'p', 'p', 'p'],
    //     ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
    // ];

    println!("Execution time: {:?}", st.elapsed());
    Ok(())
}
