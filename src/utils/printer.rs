use std::io::Write;

static PIECE_TABLE: [&str; 128] = {
    let mut table = [""; 128];
    table['r' as usize] = "♜";
    table['n' as usize] = "♞";
    table['b' as usize] = "♝";
    table['q' as usize] = "♛";
    table['k' as usize] = "♚";
    table['p' as usize] = "♟";
    table['R' as usize] = "♖";
    table['N' as usize] = "♘";
    table['B' as usize] = "♗";
    table['Q' as usize] = "♕";
    table['K' as usize] = "♔";
    table['P' as usize] = "♙";
    table
};

fn get_piece(c: char) -> Option<&'static str> {
    if (c as usize) < 128 {
        Some(PIECE_TABLE[c as usize])
    } else {
        None
    }
}

pub trait Printer: Send + Sync {
    fn print_piece(piece: char) -> String;
}

pub struct DefaultPrinter;
impl Printer for DefaultPrinter {
    fn print_piece(piece: char) -> String {
        piece.to_string()
    }
}

pub struct PrettyPrinter;
impl Printer for PrettyPrinter {
    fn print_piece(piece: char) -> String {
        get_piece(piece).unwrap_or(" ").to_string()
    }
}

pub trait View: Send + Sync {
    fn row(i: usize) -> usize;
    fn col(i: usize) -> usize;
}

pub struct WhiteView;
impl View for WhiteView {
    fn row(i: usize) -> usize {
        i
    }
    fn col(i: usize) -> usize {
        i
    }
}

pub struct BlackView;
impl View for BlackView {
    fn row(i: usize) -> usize {
        7 - i
    }
    fn col(i: usize) -> usize {
        7 - i
    }
}

pub trait AnyBoard: Send + Sync {
    fn print(&self, writer: &mut dyn Write);
    fn raw(&self) -> &[[char; 8]; 8];
}

pub fn raw_board_to_string(board: &[[char; 8]; 8]) -> String {
    let mut result = String::from("\n");
    for row in board.iter() {
        for &piece in row.iter() {
            result.push_str(&format!("{piece} "));
        }
        result.push('\n');
    }
    result
}
