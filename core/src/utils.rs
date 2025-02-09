use device_query::{DeviceQuery, DeviceState, MouseState};
use image::imageops::crop_imm;
use image::io::Reader as ImageReader;
use image::{ColorType, ExtendedColorType, ImageBuffer, Luma, Rgba};
use screenshots::Screen;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

#[derive(PartialEq)]
pub enum Color {
    WHITE,
    BLACK,
}

pub struct BoardPieces {
    pawns: (
        ImageBuffer<Luma<u8>, Vec<u8>>,
        ImageBuffer<Luma<u8>, Vec<u8>>,
    ),
    rooks: (
        ImageBuffer<Luma<u8>, Vec<u8>>,
        ImageBuffer<Luma<u8>, Vec<u8>>,
    ),
    bishops: (
        ImageBuffer<Luma<u8>, Vec<u8>>,
        ImageBuffer<Luma<u8>, Vec<u8>>,
    ),
    knights: (
        ImageBuffer<Luma<u8>, Vec<u8>>,
        ImageBuffer<Luma<u8>, Vec<u8>>,
    ),
    queens: (
        ImageBuffer<Luma<u8>, Vec<u8>>,
        ImageBuffer<Luma<u8>, Vec<u8>>,
    ),
    kings: (
        ImageBuffer<Luma<u8>, Vec<u8>>,
        ImageBuffer<Luma<u8>, Vec<u8>>,
    ),
}

impl BoardPieces {
    pub fn default(directory: &str) -> Result<Self, String> {
        Ok(Self {
            pawns: (
                BoardPieces::load_piece(directory, "pawn_white.png")?,
                BoardPieces::load_piece(directory, "pawn_black.png")?,
            ),
            rooks: (
                BoardPieces::load_piece(directory, "rook_white.png")?,
                BoardPieces::load_piece(directory, "rook_black.png")?,
            ),
            bishops: (
                BoardPieces::load_piece(directory, "bishop_white.png")?,
                BoardPieces::load_piece(directory, "bishop_black.png")?,
            ),
            knights: (
                BoardPieces::load_piece(directory, "knight_white.png")?,
                BoardPieces::load_piece(directory, "knight_black.png")?,
            ),
            queens: (
                BoardPieces::load_piece(directory, "queen_white.png")?,
                BoardPieces::load_piece(directory, "queen_black.png")?,
            ),
            kings: (
                BoardPieces::load_piece(directory, "king_white.png")?,
                BoardPieces::load_piece(directory, "king_black.png")?,
            ),
        })
    }

    fn load_piece(directory: &str, image: &str) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, String> {
        let path = Path::new(directory).join(image);
        let img = ImageReader::open(&path)
            .map_err(|_| format!("Not found {:?}", path))?
            .decode()
            .map_err(|_| format!("Decode error {:?}", path))?;
        Ok(img.to_luma8())
    }
}

#[derive(Debug, Default)]
pub struct ScreenArea {
    start_x: Option<i32>,
    start_y: Option<i32>,
    end_x: Option<i32>,
    end_y: Option<i32>,
}

fn detect_white_edges(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (u32, u32, u32, u32) {
    let (width, height) = image.dimensions();

    let mut x_min = width;
    let mut x_max = 0;
    let mut y_min = height;
    let mut y_max = 0;

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y)[0];

            if pixel > 220 {
                if x < x_min {
                    x_min = x;
                }
                if x > x_max {
                    x_max = x;
                }
                if y < y_min {
                    y_min = y;
                }
                if y > y_max {
                    y_max = y;
                }
            }
        }
    }

    (x_min, x_max, y_min, y_max)
}

pub fn trimm(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let (x_min, x_max, y_min, y_max) = detect_white_edges(image);

    let cropped = crop_imm(image, x_min, y_min, x_max - x_min, y_max - y_min).to_image();
    cropped
}

pub fn to_binary(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    threshold: u32,
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let (width, height) = image.dimensions();
    let mut binary_image = ImageBuffer::<Luma<u8>, Vec<u8>>::new(width, height);

    for (x, y, pixel) in image.enumerate_pixels() {
        let intensity = (pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3;
        let binary_value = if intensity > threshold { 255 } else { 0 };
        binary_image.put_pixel(x, y, Luma([binary_value]));
    }

    binary_image
}

pub fn get_screen_area() -> Result<ScreenArea, Box<dyn std::error::Error>> {
    let selection = Arc::new(Mutex::new(ScreenArea::default()));
    let device_state = DeviceState::new();
    let mut selecting = false;

    loop {
        let mouse: MouseState = device_state.get_mouse();
        let (x, y) = (mouse.coords.0 as i32, mouse.coords.1 as i32);

        // TODO: tmp bind with int
        if mouse.button_pressed[1] {
            if !selecting {
                let mut sel = selection.lock().unwrap();
                sel.start_x = Some(x);
                sel.start_y = Some(y);
                selecting = true;
            }
        } else if selecting {
            let mut sel = selection.lock().unwrap();
            sel.end_x = Some(x);
            sel.end_y = Some(y);
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }
    let final_selection = Arc::try_unwrap(selection).unwrap().into_inner().unwrap();

    Ok(final_selection)
}

pub fn extract_board_state(image_data: (Vec<u8>, u32, u32)) -> [[bool; 8]; 8] {
    let _vec = image_data.0;
    let width = image_data.1;
    let height = image_data.2;

    let square_width = width / 8;
    let square_height = height / 8;

    let mut board = [[false; 8]; 8];

    for row in 0..8 {
        for col in 0..8 {
            let mut black_pixel_count = 0;
            let mut all = 0;

            for y in (row * square_height)..((row + 1) * square_height) {
                for x in (col * square_width)..((col + 1) * square_width) {
                    let pixel_index = (y * width + x) as usize; //TODO
                    if _vec[pixel_index] == 0 {
                        black_pixel_count += 1;
                    }
                    all += 1;
                }
            }

            board[row as usize][col as usize] =
                (black_pixel_count * 100 / all) as f32 > 0.05 as f32;
        }
    }

    board
}

pub fn detect_move(
    before: &[[bool; 8]; 8],
    after: &[[bool; 8]; 8],
) -> ((usize, usize), (usize, usize)) {
    let mut start_pos = None;
    let mut end_pos = None;

    for row in 0..8 {
        for col in 0..8 {
            if before[row][col] && !after[row][col] {
                start_pos = Some((col, row));
            }
            if !before[row][col] && after[row][col] {
                end_pos = Some((col, row));
            }
        }
    }

    (
        start_pos.expect("Not found start piece position"),
        end_pos.expect("Not found end piece position"),
    )
}

pub fn pixel_to_chess_coord(x: usize, y: usize, board_width: usize, board_height: usize) -> String {
    let file = (b'a' + x as u8) as char; //  (a-h)
    let rank = (board_height - y).to_string(); //  (1-8)
    format!("{}{}", file, rank)
}

pub fn check_if_board_was_changed(previous: &[[bool; 8]; 8], current: &[[bool; 8]; 8]) -> bool {
    // TODO: temporary check if two positions were changed -> potential problem with promote pawns
    let mut counter = 0;
    for row in 0..8 {
        for col in 0..8 {
            if previous[row][col] != current[row][col] {
                // return true;
                counter += 1;
            }
        }
    }

    return counter == 2;
}

// pub fn runner() {
//     println!("Take a screenshot");
//     let selection = get_screen_area().unwrap();
//     let initial = take_screenshot(&selection).unwrap();
//     let width = initial.1 as usize;
//     let height = initial.2 as usize;
//     let board: [[bool; 8]; 8] = extract_board_state(initial);
//     let mut previous_board = board.clone();

//     for _ in 0..50 {
//         std::thread::sleep(std::time::Duration::from_secs(1));
//         let ndata: (Vec<u8>, u32, u32) = take_screenshot(&selection).unwrap();
//         let raw_data = ndata.clone().0;
//         let current = extract_board_state(ndata);
//         if check_if_board_was_changed(&previous_board, &current) {
//             test_save_img(&raw_data, width as u32, height as u32);
//             let (start, end) = detect_move(&previous_board, &current);
//             let start_pos = pixel_to_chess_coord(start.0, start.1, width, height);
//             let end_pos = pixel_to_chess_coord(end.0, end.1, width, height);
//             println!("{} -> {}", start_pos, end_pos);
//             // println!("any changes{}")
//             previous_board = current;
//         } else {
//             println!("Not found any changes")
//         }
//     }
// }

pub fn calculate_rms(img1: &(Vec<u8>, u32, u32), img2: &(Vec<u8>, u32, u32)) -> f64 {
    let (data1, w1, h1) = img1;
    let (data2, w2, h2) = img2;

    if w1 != w2 || h1 != h2 || data1.len() != data2.len() {
        return f64::MAX;
    }

    let sum_diff: f64 = data1
        .iter()
        .zip(data2.iter())
        .map(|(p1, p2)| (*p1 as f64 - *p2 as f64).powi(2))
        .sum();

    (sum_diff / data1.len() as f64).sqrt()
}

pub fn player_color(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> Color {
    // TODO!
    Color::WHITE
}

pub fn create_piece_templates(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
    color: Color,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = image.dimensions();
    let square_width = width / 8;
    let square_height = height / 8;

    let map: std::collections::HashMap<&str, (usize, usize)>;
    if color == Color::WHITE {
        map = std::collections::HashMap::from([
            ("rook_black", (0, 0)),
            ("rook_white", (7, 0)),
            ("knight_black", (0, 1)),
            ("knight_white", (7, 1)),
            ("bishop_black", (0, 2)),
            ("bishop_white", (7, 2)),
            ("queen_black", (0, 3)),
            ("queen_white", (7, 3)),
            ("king_black", (0, 4)),
            ("king_white", (7, 4)),
            ("pawn_black", (6, 0)),
            ("pawn_white", (1, 0)),
        ]);
    } else {
        map = std::collections::HashMap::from([
            ("rook_white", (0, 0)),
            ("rook_black", (7, 0)),
            ("knight_white", (0, 1)),
            ("knight_black", (7, 1)),
            ("bishop_white", (0, 2)),
            ("bishop_black", (7, 2)),
            ("king_white", (0, 3)),
            ("king_black", (7, 3)),
            ("queen_white", (0, 4)),
            ("queen_black", (7, 4)),
            ("pawn_white", (6, 0)),
            ("pawn_black", (1, 0)),
        ]);
    }

    for (piece, (row, col)) in map {
        let x = (col as u32) * square_width;
        let y = (row as u32) * square_height;

        let square = crop_imm(image, x, y, square_width, square_height).to_image();

        let filename = format!("templates/piece_{}.png", piece);
        square.save(filename).unwrap();
    }
    Ok(())
}

pub fn load_pieces_from_templates() -> BoardPieces {
    BoardPieces::default("templates").expect("Can not load templates")
}

pub fn take_screenshot(
    selection: &ScreenArea,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
    if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
        selection.start_x,
        selection.start_y,
        selection.end_x,
        selection.end_y,
    ) {
        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x2 - x1).abs();
        let height = (y2 - y1).abs();

        let screens = Screen::all().unwrap();
        let screen = screens.get(0).unwrap(); //TODO: works only for primary screen

        let img = screen
            .capture_area(x, y, width as u32, height as u32)
            .unwrap();

        return Ok(img);
    }
    panic!()
}

pub fn test_save_img(ndata: &[u8], width: u32, height: u32) {
    image::save_buffer(
        &Path::new("test_img.png"),
        ndata,
        width,
        height,
        ColorType::L8,
    )
    .unwrap();
}
