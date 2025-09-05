pub mod config;
pub mod error;
pub mod metric;
pub mod proto;
pub mod traces;

pub use error::SharedError;
pub use traces::init_tracing;

/// 初始化环境变量
pub fn init_env() {
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Warning: Could not load .env file: {}", e);
    }
}
