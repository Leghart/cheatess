pub mod core;
pub mod stockfish;
pub mod utils;

pub use core::engine;
pub use core::procimg;

pub use utils::logger;
pub use utils::monitor;
pub use utils::parser;

#[allow(unused_imports)]
pub use utils::printer;

static LOGGER: utils::logger::Logger = utils::logger::Logger;
