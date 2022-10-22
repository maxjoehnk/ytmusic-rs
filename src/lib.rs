pub use error::YoutubeMusicError;
pub use client::YoutubeMusicClient;
pub use requests::search::*;
pub use requests::library::*;
pub use requests::browse::*;
pub use requests::playlist::*;
pub use models::*;

mod error;
mod requests;
mod client;
mod json_ext;
mod models;
pub(crate) mod parser;
