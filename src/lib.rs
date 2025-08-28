pub mod biz;
pub mod config;
pub mod core;
pub mod data;
pub mod error;
pub mod generator;
pub mod metric;
pub mod server;
pub mod service;

use dotenvy::dotenv;

use anyhow::Result as AnyResult;
use error::TinyIdError;

pub type Result<T> = AnyResult<T, TinyIdError>;

impl From<anyhow::Error> for TinyIdError {
    fn from(err: anyhow::Error) -> Self {
        TinyIdError::InternalError(err.to_string())
    }
}

pub fn init_env() {
    dotenv().ok();
}
