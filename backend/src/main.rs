use cheatess_core::core::engine::{create_board_default, Color, DefaultPrinter};
use std::io;

fn main() {
    let board = create_board_default::<DefaultPrinter>(&Color::Black);
    board.print(&mut io::stdout());
}
