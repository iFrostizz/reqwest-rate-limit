pub mod reqwest_wrapper;

pub struct Error(pub(crate) reqwest::Error);
pub type Result<T> = std::result::Result<T, Error>;

pub struct Response(pub(crate) reqwest::Response);

pub use reqwest_wrapper::*;
