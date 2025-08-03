#[cfg(feature = "async")]
pub mod async_api;

#[cfg(not(feature = "async"))]
pub mod sync_api;

#[cfg(feature = "async")]
pub use async_api::AsyncStockfish as Stockfish;

#[cfg(not(feature = "async"))]
pub use sync_api::Stockfish;
