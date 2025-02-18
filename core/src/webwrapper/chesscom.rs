use opencv::core::Rect;
use std::collections::HashMap;

use super::ChessboardTrackerInterface;

pub struct ChesscomWrapper {
    region: Rect,
    thresholds: HashMap<String, f64>,
}

impl ChessboardTrackerInterface for ChesscomWrapper {
    fn r#type(&self) -> super::WrapperType {
        super::WrapperType::Chesscom
    }

    fn get_region(&self) -> &Rect {
        &self.region
    }

    fn get_thresholds(&self) -> &HashMap<String, f64> {
        &self.thresholds
    }
    fn pieces_path(&self) -> &'static str {
        "chesscom"
    }
}

impl Default for ChesscomWrapper {
    fn default() -> Self {
        ChesscomWrapper {
            region: Rect::new(440, 219, 758, 759),
            thresholds: HashMap::from_iter([
                ("B".to_string(), 0.35),
                ("b".to_string(), 0.55),
                ("K".to_string(), 0.2),
                ("k".to_string(), 0.3),
                ("N".to_string(), 0.1),
                ("n".to_string(), 0.3),
                ("P".to_string(), 0.15),
                ("p".to_string(), 0.9),
                ("Q".to_string(), 0.7),
                ("q".to_string(), 0.4),
                ("R".to_string(), 0.4),
                ("r".to_string(), 0.3),
            ]),
        }
    }
}
