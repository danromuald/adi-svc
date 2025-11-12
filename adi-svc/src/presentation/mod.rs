/// Presentation layer - gRPC and REST API servers
/// 
/// This layer handles incoming requests and translates them
/// to application service calls.

pub mod grpc;
pub mod rest;
pub mod converters;

pub use grpc::*;
pub use rest::*;
pub use converters::*;

