// Logic based on board operations with arrays representaitons.
// Contains functions which calculate position in board, detect last move,
// transform data to stockfish format etc.
// TODO!: handle castle + notation for stockfish
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

pub trait AnyBoard<P: Printer> {
    fn print(&self, writer: &mut dyn Write);
    fn raw(&self) -> &[[char; 8]; 8];
}

impl<P: Printer, V: View> AnyBoard<P> for Board<P, V> {
    fn print(&self, writer: &mut dyn Write) {
        Board::print(self, writer);
    }
    fn raw(&self) -> &[[char; 8]; 8] {
        &self.raw
    }
}

pub trait View {
    fn row(i: usize) -> usize;
    fn col(i: usize) -> usize;
}

pub struct WhiteView;
pub struct BlackView;

impl View for WhiteView {
    fn row(i: usize) -> usize {
        i
    }
    fn col(i: usize) -> usize {
        i
    }
}

impl View for BlackView {
    fn row(i: usize) -> usize {
        7 - i
    }
    fn col(i: usize) -> usize {
        7 - i
    }
}

pub struct Board<P: Printer, V: View> {
    pub raw: [[char; 8]; 8],
    printer: std::marker::PhantomData<(P, V)>,
}

impl<P: Printer, V: View> Board<P, V> {
    pub fn new(data: [[char; 8]; 8]) -> Self {
        Board {
            raw: data,
            printer: std::marker::PhantomData,
        }
    }

    pub const fn default_white() -> Self {
        Board {
            raw: [
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
            raw: [
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

    pub fn print<W: Write + ?Sized>(&self, writer: &mut W) {
        let transposed_board: Vec<Vec<_>> = (0..8)
            .map(|row| (0..8).map(|col| self.raw[row][col]).collect())
            .collect();

        write!(writer, "    ").unwrap();
        for j in 0..8 {
            let col = V::col(j);
            let letter = (b'a' + col as u8) as char;
            write!(writer, " {letter}  ").unwrap();
        }
        writeln!(writer).unwrap();
        writeln!(writer, "  +---+---+---+---+---+---+---+---+").unwrap();

        for (i, row) in transposed_board.iter().enumerate() {
            let _row_idx = V::row(i);
            write!(writer, "{} |", 8 - _row_idx).unwrap();
            for col in row.iter() {
                let p = P::print_piece(*col);
                let space = if p.is_empty() { "  " } else { " " };
                write!(writer, " {p}{space}|").unwrap();
            }

            writeln!(writer, " {}", 8 - _row_idx).unwrap();
            writeln!(writer, "  +---+---+---+---+---+---+---+---+").unwrap();
        }
        write!(writer, "    ").unwrap();

        for j in 0..8 {
            let col = V::col(j);
            let letter = (b'a' + col as u8) as char;
            write!(writer, " {letter}  ").unwrap();
        }
        writeln!(writer).unwrap();
    }
}

#[derive(Debug, PartialEq)]
pub enum Color {
    White,
    Black,
}

pub fn create_board<P: Printer + 'static>(player_color: &Color) -> Box<dyn AnyBoard<P>> {
    match player_color {
        Color::White => Box::new(Board::<P, WhiteView>::default_white()),
        Color::Black => Box::new(Board::<P, BlackView>::default_black()),
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
fn coords_to_position(row: usize, col: usize, player_color: &Color) -> String {
    if player_color == &Color::White {
        let file = (b'a' + col as u8) as char;
        let rank = (8 - row).to_string();
        format!("{file}{rank}")
    } else {
        let file = (b'h' - col as u8) as char;
        let rank = (row + 1).to_string();
        format!("{file}{rank}")
    }
}

pub fn detect_move(
    before: &[[char; 8]; 8],
    after: &[[char; 8]; 8],
    player_color: &Color,
) -> Option<String> {
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

                // Piece was captured by another one.
                if before[row][col] != ' ' && after[row][col] != ' ' {
                    to = Some((row, col));
                }
            }
        }
    }

    if let (Some((from_row, from_col)), Some((to_row, to_col))) = (from, to) {
        Some(format!(
            "{}{}",
            coords_to_position(from_row, from_col, player_color),
            coords_to_position(to_row, to_col, player_color)
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
    fn empty_board() -> [[char; 8]; 8] {
        [[' '; 8]; 8]
    }

    #[rstest]
    #[case(Color::White,[
                ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
                ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
                ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R']
            ])]
    #[case(Color::Black,            [
                ['R', 'N', 'B', 'K', 'Q', 'B', 'N', 'R'],
                ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
                ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
                ['r', 'n', 'b', 'k', 'q', 'b', 'n', 'r']
            ])]
    fn check_create_board(#[case] player_color: Color, #[case] result: [[char; 8]; 8]) {
        let board = create_board::<DefaultPrinter>(&player_color);
        assert_eq!(*board.raw(), result);
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
    #[case(0,0,"a8".to_string(), Color::White)]
    #[case(0, 7,"h8".to_string(), Color::White)]
    #[case(7, 0,"a1".to_string(), Color::White)]
    #[case(7, 7,"h1".to_string(), Color::White)]
    #[case(4, 4,"e4".to_string(), Color::White)]
    #[case(3, 6,"g5".to_string(), Color::White)]
    #[case(0,0,"h1".to_string(), Color::Black)]
    #[case(0, 7,"a1".to_string(), Color::Black)]
    #[case(7, 0,"h8".to_string(), Color::Black)]
    #[case(7, 7,"a8".to_string(), Color::Black)]
    #[case(4, 4,"d5".to_string(), Color::Black)]
    #[case(3, 6,"b4".to_string(), Color::Black)]
    fn correct_change_coords_to_position(
        #[case] row: usize,
        #[case] col: usize,
        #[case] pos: String,
        #[case] player_color: Color,
    ) {
        let result = coords_to_position(row, col, &player_color);

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
    ],"e2e4".to_string(),Color::White, Board::<DefaultPrinter, WhiteView>::default_white())]
    #[case([
        ['r', ' ', 'b', 'q', 'k', 'b', 'n', 'r'],
        ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
        [' ', ' ', 'n', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
        ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
    ],"b8c6".to_string(),Color::White, Board::<DefaultPrinter, WhiteView>::default_white())]
    #[case([
        ['R', 'N', 'B', 'K', 'Q', 'B', 'N', 'R'],
        ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', 'n', ' ', ' ', ' ', ' ', ' '],
        ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
        ['r', ' ', 'b', 'k', 'q', 'b', 'n', 'r'],
    ],"g8f6".to_string(),Color::Black,Board::<DefaultPrinter,BlackView>::default_black())]
    fn detect_simple_move(
        #[case] after_move: [[char; 8]; 8],
        #[case] _move: String,
        #[case] player_color: Color,
        #[case] init_board: Board<DefaultPrinter, impl View>,
    ) {
        let result = detect_move(&init_board.raw, &after_move, &player_color);

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
    ],"h7h5".to_string(), Color::White)]
    #[case([
        ['R', 'N', 'B', 'K', 'Q', 'B', 'N', 'R'],
        ['P', 'P', 'P', 'P', 'P', ' ', 'P', 'P'],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', 'q', ' ', 'P', ' ', 'p', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', 'P', ' ', ' ', ' '],
        ['p', ' ', 'p', 'p', 'p', 'p', 'p', 'p'],
        ['r', 'n', 'b', ' ', 'q', 'k', ' ', ' ']
    ],[
        ['R', 'N', 'B', 'K', 'Q', 'B', 'N', 'R'],
        ['P', 'P', 'P', 'P', 'P', ' ', 'P', 'P'],
        [' ', ' ', ' ', ' ', ' ', 'p', ' ', ' '],
        [' ', 'q', ' ', 'P', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', 'P', ' ', ' ', ' '],
        ['p', ' ', 'p', 'p', 'p', 'p', 'p', 'p'],
        ['r', 'n', 'b', ' ', 'q', 'k', ' ', ' ']
    ],"c4c3".to_string(),Color::Black)]
    fn detect_complex_move(
        #[case] before_move: [[char; 8]; 8],
        #[case] after_move: [[char; 8]; 8],
        #[case] _move: String,
        #[case] player_color: Color,
    ) {
        let result = detect_move(&before_move, &after_move, &player_color);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), _move);
    }

    #[rstest]
    fn show_board_with_pieces() {
        let mut buf = Vec::new();
        let board = Board::<PrettyPrinter, BlackView>::default_black();
        board.print(&mut buf);

        let output = String::from_utf8(buf).unwrap();

        let predicted = "     h   g   f   e   d   c   b   a  
  +---+---+---+---+---+---+---+---+
1 | ♖ | ♘ | ♗ | ♔ | ♕ | ♗ | ♘ | ♖ | 1
  +---+---+---+---+---+---+---+---+
2 | ♙ | ♙ | ♙ | ♙ | ♙ | ♙ | ♙ | ♙ | 2
  +---+---+---+---+---+---+---+---+
3 |   |   |   |   |   |   |   |   | 3
  +---+---+---+---+---+---+---+---+
4 |   |   |   |   |   |   |   |   | 4
  +---+---+---+---+---+---+---+---+
5 |   |   |   |   |   |   |   |   | 5
  +---+---+---+---+---+---+---+---+
6 |   |   |   |   |   |   |   |   | 6
  +---+---+---+---+---+---+---+---+
7 | ♟ | ♟ | ♟ | ♟ | ♟ | ♟ | ♟ | ♟ | 7
  +---+---+---+---+---+---+---+---+
8 | ♜ | ♞ | ♝ | ♚ | ♛ | ♝ | ♞ | ♜ | 8
  +---+---+---+---+---+---+---+---+
     h   g   f   e   d   c   b   a  \n"
            .to_string();
        assert_eq!(output, predicted);
    }

    #[rstest]
    fn show_board_with_letters() {
        let mut buf = Vec::new();
        let board = Board::<DefaultPrinter, WhiteView>::default_white();
        board.print(&mut buf);

        let output = String::from_utf8(buf).unwrap();
        let predicted = "     a   b   c   d   e   f   g   h  
  +---+---+---+---+---+---+---+---+
8 | r | n | b | q | k | b | n | r | 8
  +---+---+---+---+---+---+---+---+
7 | p | p | p | p | p | p | p | p | 7
  +---+---+---+---+---+---+---+---+
6 |   |   |   |   |   |   |   |   | 6
  +---+---+---+---+---+---+---+---+
5 |   |   |   |   |   |   |   |   | 5
  +---+---+---+---+---+---+---+---+
4 |   |   |   |   |   |   |   |   | 4
  +---+---+---+---+---+---+---+---+
3 |   |   |   |   |   |   |   |   | 3
  +---+---+---+---+---+---+---+---+
2 | P | P | P | P | P | P | P | P | 2
  +---+---+---+---+---+---+---+---+
1 | R | N | B | Q | K | B | N | R | 1
  +---+---+---+---+---+---+---+---+
     a   b   c   d   e   f   g   h  \n";

        assert_eq!(output, predicted);
    }
}
