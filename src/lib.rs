pub mod storage;
pub mod crypto;
pub mod database;
pub mod api;
pub mod error;

pub use database::SimpleDB;
pub use error::DatabaseError;
pub use storage::{Record, Table, Value};

/// 数据库配置
#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: String,
    pub encryption_key: Option<Vec<u8>>,
    pub max_file_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            encryption_key: None,
            max_file_size: 1024 * 1024 * 10, // 10MB
        }
    }
} 