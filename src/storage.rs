use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::crypto::Crypto;
use crate::error::{DatabaseError, Result};

/// 数据记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Record {
    pub id: String,
    pub data: HashMap<String, Value>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Record {
    pub fn new(data: HashMap<String, Value>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id: Uuid::new_v4().to_string(),
            data,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, data: HashMap<String, Value>) {
        self.data = data;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// 支持的数据类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }
}

/// 表结构
#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub file_path: PathBuf,
    pub crypto: Option<Crypto>,
    records: HashMap<String, Record>,
    is_dirty: bool,
}

impl Table {
    /// 创建新表
    pub fn new(name: String, data_dir: &Path, crypto: Option<Crypto>) -> Result<Self> {
        let file_path = data_dir.join(format!("{}.db", name));
        
        let mut table = Self {
            name,
            file_path,
            crypto,
            records: HashMap::new(),
            is_dirty: false,
        };

        // 如果文件存在，加载数据
        if table.file_path.exists() {
            table.load()?;
        }

        Ok(table)
    }

    /// 插入记录
    pub fn insert(&mut self, record: Record) -> Result<String> {
        if self.records.contains_key(&record.id) {
            return Err(DatabaseError::DuplicateKey(record.id));
        }

        let id = record.id.clone();
        self.records.insert(id.clone(), record);
        self.is_dirty = true;

        Ok(id)
    }

    /// 根据ID查找记录
    pub fn find_by_id(&self, id: &str) -> Option<&Record> {
        self.records.get(id)
    }

    /// 更新记录
    pub fn update(&mut self, id: &str, data: HashMap<String, Value>) -> Result<()> {
        match self.records.get_mut(id) {
            Some(record) => {
                record.update(data);
                self.is_dirty = true;
                Ok(())
            }
            None => Err(DatabaseError::RecordNotFound(id.to_string())),
        }
    }

    /// 删除记录
    pub fn delete(&mut self, id: &str) -> Result<()> {
        match self.records.remove(id) {
            Some(_) => {
                self.is_dirty = true;
                Ok(())
            }
            None => Err(DatabaseError::RecordNotFound(id.to_string())),
        }
    }

    /// 查询所有记录
    pub fn find_all(&self) -> Vec<&Record> {
        self.records.values().collect()
    }

    /// 根据条件查询记录
    pub fn find_where<F>(&self, predicate: F) -> Vec<&Record>
    where
        F: Fn(&Record) -> bool,
    {
        self.records.values().filter(|r| predicate(r)).collect()
    }

    /// 保存到文件
    pub fn save(&mut self) -> Result<()> {
        if !self.is_dirty {
            return Ok(());
        }

        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 序列化数据
        let data = bincode::serialize(&self.records)?;

        // 加密（如果启用）
        let final_data = match &self.crypto {
            Some(crypto) => crypto.encrypt(&data)?,
            None => data,
        };

        // 写入文件
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.file_path)?;
        
        let mut writer = BufWriter::new(file);
        writer.write_all(&final_data)?;
        writer.flush()?;

        self.is_dirty = false;
        Ok(())
    }

    /// 从文件加载
    fn load(&mut self) -> Result<()> {
        let file = File::open(&self.file_path)?;
        let mut reader = BufReader::new(file);
        
        // 读取所有数据
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        if buffer.is_empty() {
            return Ok(());
        }

        // 解密（如果启用）
        let data = match &self.crypto {
            Some(crypto) => crypto.decrypt(&buffer)?,
            None => buffer,
        };

        // 反序列化
        self.records = bincode::deserialize(&data)?;
        self.is_dirty = false;

        Ok(())
    }

    /// 获取记录数量
    pub fn count(&self) -> usize {
        self.records.len()
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        if self.is_dirty {
            let _ = self.save();
        }
    }
} 