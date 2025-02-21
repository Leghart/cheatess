// Logic based on board operations with arrays representaitons.
// Contains functions which calculate position in board, detect last move,
// transform data to stockfish format etc.

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

// Insert piece to array board, based on top left position.
// TODO: add validations
pub fn register_piece(
    point: (i32, i32),
    board_size: (i32, i32),
    piece: &char,
    board: &mut [[char; 8]; 8],
) {
    let tile_width = board_size.0 / 8;
    let tile_height = board_size.1 / 8;

    let col = (point.0 / tile_width).clamp(0, 7);
    let row = (7 - (point.1 / tile_height)).clamp(0, 7);

    board[row as usize][col as usize] = *piece;
}

fn pos_to_algebraic(row: usize, col: usize) -> String {
    let file = (b'a' + col as u8) as char;
    let rank = (8 - row).to_string();
    format!("{}{}", file, rank)
}

pub fn detect_move(before: [[char; 8]; 8], after: [[char; 8]; 8]) -> Option<String> {
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
        Some(format!(
            "{}{}",
            pos_to_algebraic(from_row, from_col),
            pos_to_algebraic(to_row, to_col)
        ))
    } else {
        None
    }
}
