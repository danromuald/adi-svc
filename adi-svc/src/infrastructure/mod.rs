/// Infrastructure layer - External service adapters
/// 
/// This layer contains implementations of ports that interact with
/// external services like Azure AI Document Intelligence.

pub mod azure;
pub mod storage;
pub mod tracker;
pub mod postgres_tracker;
pub mod config;

pub use azure::*;
pub use storage::*;
pub use tracker::*;
pub use postgres_tracker::*;
pub use config::*;

