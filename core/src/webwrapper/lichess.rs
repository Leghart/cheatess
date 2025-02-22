use opencv::core::Rect;
use std::collections::HashMap;

use super::ChessboardTrackerInterface;

pub struct LichessWrapper {
    region: Rect,
    thresholds: HashMap<char, f64>,
}

impl ChessboardTrackerInterface for LichessWrapper {
    fn new(region: Rect, thresholds: HashMap<char, f64>) -> Self {
        LichessWrapper { region, thresholds }
    }

    fn mode(&self) -> super::WrapperMode {
        super::WrapperMode::Lichess
    }

    fn get_region(&self) -> &Rect {
        &self.region
    }

    fn get_thresholds(&self) -> &HashMap<char, f64> {
        &self.thresholds
    }
    fn pieces_path(&self) -> &'static str {
        "lichess"
    }
}

impl Default for LichessWrapper {
    fn default() -> Self {
        LichessWrapper {
            region: Rect::new(568, 218, 720, 720),
            thresholds: HashMap::from_iter([
                ('B', 0.25),
                ('b', 0.25),
                ('K', 0.2),
                ('k', 0.3),
                ('N', 0.15),
                ('n', 0.1),
                ('P', 0.1),
                ('p', 0.55),
                ('Q', 0.3),
                ('q', 0.1),
                ('R', 0.05),
                ('r', 0.3),
            ]),
        }
    }
}
