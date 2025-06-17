use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("加密错误: {0}")]
    Encryption(String),

    #[error("表不存在: {0}")]
    TableNotFound(String),

    #[error("记录不存在: {0}")]
    RecordNotFound(String),

    #[error("重复的键: {0}")]
    DuplicateKey(String),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("数据格式错误: {0}")]
    DataFormat(String),
}

pub type Result<T> = std::result::Result<T, DatabaseError>; 