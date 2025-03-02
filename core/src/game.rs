use super::config::Config;

pub enum PlayAs {
    WHITE,
    BLACK,
}

// Use already configured app to play a game.
pub struct Game {
    color: PlayAs,
    config: Config,
}

impl Game {
    fn detect_player_color(&mut self) -> () {}
}
