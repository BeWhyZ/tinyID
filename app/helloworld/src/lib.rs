pub mod biz;
pub mod core;
pub mod data;
pub mod error;
pub mod server;
pub mod service;

pub use error::TinyIdError;

use anyhow::Result as AnyResult;

pub type Result<T> = AnyResult<T, TinyIdError>;

impl From<anyhow::Error> for TinyIdError {
    fn from(err: anyhow::Error) -> Self {
        TinyIdError::InternalError(err.to_string())
    }
}
