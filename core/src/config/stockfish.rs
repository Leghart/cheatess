#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct StockfishConfig {
    engine_path: String,
    depth: u8,
    memory: usize,
    elo: usize,
}

impl StockfishConfig {
    // Create config from passed data.
    pub fn new(
        engine_path: &str,
        depth: Option<u8>,
        memory: Option<usize>,
        elo: Option<usize>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(StockfishConfig {
            engine_path: engine_path.to_string(),
            depth: depth.unwrap_or(10),
            memory: memory.unwrap_or(1024),
            elo: elo.unwrap_or(2500),
        })
    }

    // Load already created config from file.
    pub fn from_cache() -> Result<Self, Box<dyn std::error::Error>> {
        let path = ".stockfish.config.json";
        let config: StockfishConfig = serde_json::from_str(path)
            .expect("Can not serialize deserialize stockfish config object!");

        Ok(config)
    }
}
