use opencv::core::Rect;
use std::collections::HashMap;

use super::ChessboardTrackerInterface;

pub struct LichessWrapper {
    region: Rect,
    thresholds: HashMap<String, f64>,
}

impl ChessboardTrackerInterface for LichessWrapper {
    fn r#type(&self) -> super::WrapperType {
        super::WrapperType::Lichess
    }

    fn get_region(&self) -> &Rect {
        &self.region
    }

    fn get_thresholds(&self) -> &HashMap<String, f64> {
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
                ("B".to_string(), 0.25),
                ("b".to_string(), 0.25),
                ("K".to_string(), 0.2),
                ("k".to_string(), 0.3),
                ("N".to_string(), 0.15),
                ("n".to_string(), 0.1),
                ("P".to_string(), 0.1),
                ("p".to_string(), 0.55),
                ("Q".to_string(), 0.3),
                ("q".to_string(), 0.1),
                ("R".to_string(), 0.05),
                ("r".to_string(), 0.3),
            ]),
        }
    }
}
