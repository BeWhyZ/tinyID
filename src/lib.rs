pub mod config;
pub mod core;
pub mod error;
pub mod generator;
pub mod metric;
pub mod server;
pub mod service;

use dotenvy::dotenv;

use error::TinyIdError;

pub type Result<T> = std::result::Result<T, TinyIdError>;

fn init_env() {
    dotenv().ok();
}
