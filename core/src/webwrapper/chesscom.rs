use crate::utils::screen_region::ScreenRegion;
use std::collections::HashMap;

use super::ChessboardTrackerInterface;

pub struct ChesscomWrapper {
    region: ScreenRegion,
    thresholds: HashMap<char, f64>,
}

impl ChessboardTrackerInterface for ChesscomWrapper {
    fn new(region: ScreenRegion, thresholds: HashMap<char, f64>) -> Self {
        ChesscomWrapper { region, thresholds }
    }

    fn mode(&self) -> super::WrapperMode {
        super::WrapperMode::Chesscom
    }

    fn get_region(&self) -> &ScreenRegion {
        &self.region
    }

    fn get_thresholds(&self) -> &HashMap<char, f64> {
        &self.thresholds
    }
    fn pieces_path(&self) -> &'static str {
        "chesscom"
    }
}

impl Default for ChesscomWrapper {
    fn default() -> Self {
        ChesscomWrapper {
            region: ScreenRegion::new(440, 219, 758, 759),
            thresholds: HashMap::from_iter([
                ('B', 0.35),
                ('b', 0.55),
                ('K', 0.2),
                ('k', 0.3),
                ('N', 0.1),
                ('n', 0.3),
                ('P', 0.15),
                ('p', 0.9),
                ('Q', 0.7),
                ('q', 0.5),
                ('R', 0.4),
                ('r', 0.3),
            ]),
        }
    }
}
