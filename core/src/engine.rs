// Logic based on board operations with arrays representaitons.
// Contains functions which calculate position in board, detect last move,
// transform data to stockfish format etc.
// TODO: every func & test is for white view

static PIECE_TABLE: [&'static str; 128] = {
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
        let v = PIECE_TABLE[c as usize];
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    } else {
        None
    }
}

pub trait Printer {
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

pub struct Board<P: Printer> {
    pub board: [[char; 8]; 8],
    printer: std::marker::PhantomData<P>,
}

impl<P: Printer> Board<P> {
    pub fn new(data: [[char; 8]; 8]) -> Self {
        Board {
            board: data,
            printer: std::marker::PhantomData,
        }
    }

    pub const fn default_white() -> Self {
        Board {
            board: [
                ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
                ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
                ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
            ],
            printer: std::marker::PhantomData,
        }
    }

    pub const fn default_black() -> Self {
        Board {
            board: [
                ['R', 'N', 'B', 'K', 'Q', 'B', 'N', 'R'],
                ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
                ['r', 'n', 'b', 'k', 'q', 'b', 'n', 'r'],
            ],
            printer: std::marker::PhantomData,
        }
    }

    pub fn print(&self) {
        let transposed_board: Vec<Vec<_>> = (0..8)
            .map(|row| (0..8).map(|col| self.board[row][col]).collect())
            .collect();

        println!("+---+---+---+---+---+---+---+---+");

        for row in transposed_board.iter() {
            print!("|");
            for col in row.iter() {
                print!(" {} |", P::print_piece(*col));
            }
            println!();
            println!("+---+---+---+---+---+---+---+---+");
        }
    }
}

#[derive(Debug)]
pub enum Color {
    White,
    Black,
}

pub fn create_board<P: Printer>(player_color: &Color) -> Board<P> {
    match player_color {
        Color::White => Board::<P>::default_white(),
        Color::Black => Board::<P>::default_black(),
    }
}

// Insert piece to array board, based on top left position.
pub fn register_piece(
    point: (i32, i32),
    board_size: (i32, i32),
    piece: char,
    board: &mut [[char; 8]; 8],
) -> Result<(), Box<dyn std::error::Error>> {
    let tile_width = board_size.0 / 8;
    let tile_height = board_size.1 / 8;

    let row = (point.1 / tile_height).clamp(0, 7) as usize;
    let col = (point.0 / tile_width).clamp(0, 7) as usize;

    board[col][row] = piece;
    Ok(())
}

// Change (x,y) coordiantes to string position representation.
// TODO: add color handling (now only for whites)
fn coords_to_position(row: usize, col: usize) -> String {
    let file = (b'a' + col as u8) as char;
    let rank = (8 - row).to_string();
    format!("{}{}", file, rank)
}

pub fn detect_move(before: &[[char; 8]; 8], after: &[[char; 8]; 8]) -> Option<String> {
    let mut from: Option<(usize, usize)> = None;
    let mut to: Option<(usize, usize)> = None;

    for row in 0..8 {
        for col in 0..8 {
            if before[row][col] != after[row][col] {
                // Previously piece was at (row, col) in `before` and now this place
                // is empty.
                if before[row][col] != ' ' && after[row][col] == ' ' {
                    from = Some((row, col));
                }

                // Piece moved from empty place to a new one.
                if before[row][col] == ' ' && after[row][col] != ' ' {
                    to = Some((row, col));
                }

                // piece was captured by another one.
                if before[row][col] != ' ' && after[row][col] != ' ' {
                    to = Some((row, col));
                }
            }
        }
    }

    if let (Some((from_row, from_col)), Some((to_row, to_col))) = (from, to) {
        Some(format!(
            "{}{}",
            coords_to_position(from_row, from_col),
            coords_to_position(to_row, to_col)
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn init_board() -> [[char; 8]; 8] {
        [
            ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
            ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
            ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
        ]
    }

    #[fixture]
    fn empty_board() -> [[char; 8]; 8] {
        [[' '; 8]; 8]
    }

    #[rstest]
    fn default_white_as_init_board(init_board: [[char; 8]; 8]) {
        let board = Board::<DefaultPrinter>::default_white();
        assert_eq!(board.board, init_board);
    }

    #[rstest]
    #[case((0,0),(800,800),(0,0))] //top left piece
    #[case((0,315),(360,360),(0,7))] //top right piece
    #[case((315,0),(360,360),(7,0))] //bottom left piece
    #[case((700,700),(800,800),(7,7))] //bottom right piece
    #[case((180,135),(360,360),(4,3))] //d4 piece

    fn register_piece_correct_insert(
        #[case] point: (i32, i32),
        #[case] board_width_height: (i32, i32),
        #[case] result_row_col: (usize, usize),
        mut empty_board: [[char; 8]; 8],
    ) {
        assert!(register_piece(point, board_width_height, 'X', &mut empty_board).is_ok());
        assert_eq!(empty_board[result_row_col.0][result_row_col.1], 'X');
    }

    #[rstest]
    #[case(0,0,"a8".to_string())]
    #[case(0, 7,"h8".to_string())]
    #[case(7, 0,"a1".to_string())]
    #[case(7, 7,"h1".to_string())]
    #[case(4, 4,"e4".to_string())]
    #[case(3, 6,"g5".to_string())]
    fn correct_change_coords_to_position(
        #[case] row: usize,
        #[case] col: usize,
        #[case] pos: String,
    ) {
        let result = coords_to_position(row, col);

        assert_eq!(result, pos);
    }

    #[rstest]
    #[case([
        ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
        ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', 'P', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        ['P', 'P', 'P', 'P', ' ', 'P', 'P', 'P'],
        ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
    ],"e2e4".to_string())]
    #[case([
        ['r', ' ', 'b', 'q', 'k', 'b', 'n', 'r'],
        ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
        [' ', ' ', 'n', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
        ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
    ],"b8c6".to_string())]
    fn detect_simple_move(
        #[case] after_move: [[char; 8]; 8],
        #[case] _move: String,
        init_board: [[char; 8]; 8],
    ) {
        let result = detect_move(&init_board, &after_move);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), _move);
    }

    #[rstest]
    #[case([
        ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
        ['p', 'p', ' ', ' ', 'p', 'p', 'p', 'p'],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', 'P', ' ', ' ', ' ', ' ', 'N', ' '],
        [' ', ' ', 'p', 'q', 'P', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        ['P', ' ', 'P', ' ', ' ', 'P', ' ', 'P'],
        ['R', 'N', ' ', 'Q', 'K', 'B', 'N', 'R'],
    ],[
        ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
        ['p', 'p', ' ', ' ', 'p', 'p', 'p', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', 'P', ' ', ' ', ' ', ' ', 'N', 'p'],
        [' ', ' ', 'p', 'q', 'P', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        ['P', ' ', 'P', ' ', ' ', 'P', ' ', 'P'],
        ['R', 'N', ' ', 'Q', 'K', 'B', 'N', 'R'],
    ],"h7h5".to_string())]
    #[case([
        ['r', ' ', 'b', 'q', 'k', 'b', 'n', 'r'],
        ['p', 'P', ' ', ' ', 'p', 'p', 'p', 'p'],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', 'N', ' '],
        [' ', ' ', 'p', 'q', 'P', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        ['P', ' ', 'P', ' ', ' ', 'P', ' ', 'P'],
        ['R', 'N', ' ', 'Q', 'K', 'B', 'N', 'R'],
    ],[
        ['r', 'Q', 'b', 'q', 'k', 'b', 'n', 'r'],
        ['p', ' ', ' ', ' ', 'p', 'p', 'p', 'p'],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', 'N', ' '],
        [' ', ' ', 'p', 'q', 'P', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        ['P', ' ', 'P', ' ', ' ', 'P', ' ', 'P'],
        ['R', 'N', ' ', 'Q', 'K', 'B', 'N', 'R'],
    ],"b7b8".to_string())]
    fn detect_complex_move(
        #[case] before_move: [[char; 8]; 8],
        #[case] after_move: [[char; 8]; 8],
        #[case] _move: String,
    ) {
        let result = detect_move(&before_move, &after_move);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), _move);
    }
}
