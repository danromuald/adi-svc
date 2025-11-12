/// adi-svc library
/// 
/// This crate provides a Rust wrapper for Azure AI Document Intelligence
/// with both gRPC and REST interfaces.

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;
pub mod generated;

pub use domain::*;
pub use application::*;
pub use infrastructure::*;
pub use presentation::*;

