extern crate opencv;

mod engine;
mod image;
use image::ImageProcessing;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let st = Instant::now();
    let mut proc = ImageProcessing::default();
    proc.load_piece_images("working_pieces")?;
    let img = proc.load_image("boards/board.jpg")?;

    // proc.load_piece_images("my_pieces")?;
    // let img = proc.load_image("boards/ccc.png")?;

    let board = proc.image_to_board(&img);

    println!("AAAA");
    for row in board.iter() {
        println!("{:?}", row);
    }

    println!("Execution time: {:?}", st.elapsed());
    Ok(())
}
