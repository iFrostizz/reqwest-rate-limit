//! Wrapper for reqwest which will allow us to write a custom middleware for requests and responses.

mod client;
mod middleware;
mod request_builder;

pub use client::Client;
pub use middleware::Middleware;
pub use request_builder::RequestBuilder;
