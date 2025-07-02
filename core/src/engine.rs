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

pub struct Board {
    pub board: [[char; 8]; 8],
}

impl Board {
    pub fn new(data: [[char; 8]; 8]) -> Self {
        Board { board: data }
    }

    pub const fn default_white() -> Self {
        Board {
            board: [
                ['r', 'p', ' ', ' ', ' ', ' ', 'P', 'R'],
                ['n', 'p', ' ', ' ', ' ', ' ', 'P', 'N'],
                ['b', 'p', ' ', ' ', ' ', ' ', 'P', 'B'],
                ['q', 'p', ' ', ' ', ' ', ' ', 'P', 'Q'],
                ['k', 'p', ' ', ' ', ' ', ' ', 'P', 'K'],
                ['b', 'p', ' ', ' ', ' ', ' ', 'P', 'B'],
                ['n', 'p', ' ', ' ', ' ', ' ', 'P', 'N'],
                ['r', 'p', ' ', ' ', ' ', ' ', 'P', 'R'],
            ],
        }
    }

    pub const fn default_black() -> Self {
        Board {
            board: [
                ['R', 'P', ' ', ' ', ' ', ' ', 'p', 'r'],
                ['N', 'P', ' ', ' ', ' ', ' ', 'p', 'n'],
                ['B', 'P', ' ', ' ', ' ', ' ', 'p', 'b'],
                ['K', 'P', ' ', ' ', ' ', ' ', 'p', 'k'],
                ['Q', 'P', ' ', ' ', ' ', ' ', 'p', 'q'],
                ['B', 'P', ' ', ' ', ' ', ' ', 'p', 'b'],
                ['N', 'P', ' ', ' ', ' ', ' ', 'p', 'n'],
                ['R', 'P', ' ', ' ', ' ', ' ', 'p', 'r'],
            ],
        }
    }

    pub fn print(&self) {
        let transposed_board: Vec<Vec<_>> = (0..8)
            .map(|col| (0..8).map(|row| self.board[row][col]).collect())
            .collect();

        println!("+---+---+---+---+---+---+---+---+");

        for row in transposed_board.iter() {
            print!("|");
            for col in row.iter() {
                print!(" {} |", get_piece(*col).unwrap_or(" "));
            }
            println!();
            println!("+---+---+---+---+---+---+---+---+");
        }
    }
}

// Insert piece to array board, based on top left position.
// TODO: add validations
pub fn register_piece(
    point: (i32, i32),
    board_size: (i32, i32),
    piece: char,
    board: &mut [[char; 8]; 8],
) -> Result<(), Box<dyn std::error::Error>> {
    let tile_width = board_size.0 / 8;
    let tile_height = board_size.1 / 8;

    let row = (point.0 / tile_height).clamp(0, 7) as usize;
    let col = (point.1 / tile_width).clamp(0, 7) as usize;

    board[row][col] = piece;
    Ok(())
}

// Change (x,y) coordiantes to string position representation.
// TODO: add validations
// TODO: add color handling (now only for whites)
fn coords_to_position(row: usize, col: usize) -> Result<String, Box<dyn std::error::Error>> {
    let file = (b'a' + col as u8) as char;
    let rank = (8 - row).to_string();
    Ok(format!("{}{}", file, rank))
}

pub fn detect_move(
    before: &[[char; 8]; 8],
    after: &[[char; 8]; 8],
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut from: Option<(usize, usize)> = None;
    let mut to: Option<(usize, usize)> = None;

    for row in 0..8 {
        for col in 0..8 {
            if before[row][col] != after[row][col] {
                if before[row][col] != ' ' && after[row][col] == ' ' {
                    from = Some((row, col));
                }
                if before[row][col] == ' ' && after[row][col] != ' ' {
                    to = Some((row, col));
                }
            }
        }
    }

    if let (Some((from_row, from_col)), Some((to_row, to_col))) = (from, to) {
        Ok(Some(format!(
            "{}{}",
            coords_to_position(from_row, from_col)?,
            coords_to_position(to_row, to_col)?
        )))
    } else {
        Ok(None)
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
    #[case((0,0),(800,800),(0,0))] //top left piece
    #[case((0,315),(360,360),(0,7))] //top right piece
    #[case((315,0),(360,360),(7,0))] //bottom left piece
    #[case((700,700),(800,800),(7,7))] //bottom right piece
    #[case((180,180),(360,360),(4,4))] //e4 piece

    fn test_register_piece_correct_insert(
        #[case] point: (i32, i32),
        #[case] board_size: (i32, i32),
        #[case] result_row_col: (usize, usize),
        mut empty_board: [[char; 8]; 8],
    ) {
        assert!(register_piece(point, board_size, 'X', &mut empty_board).is_ok());
        assert_eq!(empty_board[result_row_col.0][result_row_col.1], 'X');
    }

    #[rstest]
    #[case(0,0,"a8".to_string())]
    #[case(0, 7,"h8".to_string())]
    #[case(7, 0,"a1".to_string())]
    #[case(7, 7,"h1".to_string())]
    #[case(4, 4,"e4".to_string())]
    #[case(3, 6,"g5".to_string())]
    fn test_coords_to_position(#[case] row: usize, #[case] col: usize, #[case] pos: String) {
        let result = coords_to_position(row, col);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), pos);
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
    fn test_detect_simple_move(
        #[case] after_move: [[char; 8]; 8],
        #[case] _move: String,
        init_board: [[char; 8]; 8],
    ) {
        let result = detect_move(&init_board, &after_move);

        assert!(result.is_ok());

        let _str = result.unwrap();
        assert!(_str.is_some());
        assert_eq!(_str.unwrap(), _move);
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
    fn test_detect_complex_move(
        #[case] before_move: [[char; 8]; 8],
        #[case] after_move: [[char; 8]; 8],
        #[case] _move: String,
    ) {
        let result = detect_move(&before_move, &after_move);

        assert!(result.is_ok());

        let _str = result.unwrap();
        assert!(_str.is_some());
        assert_eq!(_str.unwrap(), _move);
    }
}
