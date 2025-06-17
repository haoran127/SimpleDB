use std::collections::HashMap;
use std::path::PathBuf;

use crate::crypto::Crypto;
use crate::error::{DatabaseError, Result};
use crate::storage::{Record, Table, Value};
use crate::Config;

/// 简单数据库
pub struct SimpleDB {
    config: Config,
    tables: HashMap<String, Table>,
    crypto: Option<Crypto>,
}

impl SimpleDB {
    /// 创建新的数据库实例
    pub fn new(config: Config) -> Result<Self> {
        // 创建数据目录
        std::fs::create_dir_all(&config.data_dir)?;

        // 初始化加密器
        let crypto = if let Some(key) = &config.encryption_key {
            Some(Crypto::new(key)?)
        } else {
            None
        };

        let mut db = Self {
            config,
            tables: HashMap::new(),
            crypto,
        };

        // 自动加载现有的表
        db.load_existing_tables()?;

        Ok(db)
    }

    /// 加载现有的表文件
    fn load_existing_tables(&mut self) -> Result<()> {
        let data_dir = PathBuf::from(&self.config.data_dir);
        
        // 检查数据目录是否存在
        if !data_dir.exists() {
            return Ok(());
        }

        // 扫描.db文件
        for entry in std::fs::read_dir(data_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extension == "db" {
                    if let Some(stem) = path.file_stem() {
                        if let Some(table_name) = stem.to_str() {
                            // 加载表
                            let data_dir = PathBuf::from(&self.config.data_dir);
                            let table = Table::new(table_name.to_string(), &data_dir, self.crypto.clone())?;
                            self.tables.insert(table_name.to_string(), table);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 创建表
    pub fn create_table(&mut self, name: &str) -> Result<()> {
        if self.tables.contains_key(name) {
            return Ok(()); // 表已存在，直接返回
        }

        let data_dir = PathBuf::from(&self.config.data_dir);
        let table = Table::new(name.to_string(), &data_dir, self.crypto.clone())?;
        self.tables.insert(name.to_string(), table);

        Ok(())
    }

    /// 删除表
    pub fn drop_table(&mut self, name: &str) -> Result<()> {
        if let Some(table) = self.tables.remove(name) {
            // 删除表文件
            if table.file_path.exists() {
                std::fs::remove_file(&table.file_path)?;
            }
        }
        Ok(())
    }

    /// 获取表
    fn get_table(&self, name: &str) -> Result<&Table> {
        self.tables
            .get(name)
            .ok_or_else(|| DatabaseError::TableNotFound(name.to_string()))
    }

    /// 获取可变表
    fn get_table_mut(&mut self, name: &str) -> Result<&mut Table> {
        self.tables
            .get_mut(name)
            .ok_or_else(|| DatabaseError::TableNotFound(name.to_string()))
    }

    /// 插入记录
    pub fn insert(&mut self, table_name: &str, data: HashMap<String, Value>) -> Result<String> {
        // 如果表不存在，自动创建
        if !self.tables.contains_key(table_name) {
            self.create_table(table_name)?;
        }

        let record = Record::new(data);
        let table = self.get_table_mut(table_name)?;
        table.insert(record)
    }

    /// 根据ID查找记录
    pub fn find_by_id(&self, table_name: &str, id: &str) -> Result<Option<&Record>> {
        let table = self.get_table(table_name)?;
        Ok(table.find_by_id(id))
    }

    /// 更新记录
    pub fn update(
        &mut self,
        table_name: &str,
        id: &str,
        data: HashMap<String, Value>,
    ) -> Result<()> {
        let table = self.get_table_mut(table_name)?;
        table.update(id, data)
    }

    /// 删除记录
    pub fn delete(&mut self, table_name: &str, id: &str) -> Result<()> {
        let table = self.get_table_mut(table_name)?;
        table.delete(id)
    }

    /// 查询所有记录
    pub fn find_all(&self, table_name: &str) -> Result<Vec<&Record>> {
        let table = self.get_table(table_name)?;
        Ok(table.find_all())
    }

    /// 根据条件查询记录
    pub fn find_where<F>(&self, table_name: &str, predicate: F) -> Result<Vec<&Record>>
    where
        F: Fn(&Record) -> bool,
    {
        let table = self.get_table(table_name)?;
        Ok(table.find_where(predicate))
    }

    /// 保存所有表到磁盘
    pub fn save_all(&mut self) -> Result<()> {
        for table in self.tables.values_mut() {
            table.save()?;
        }
        Ok(())
    }

    /// 获取表列表
    pub fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// 获取表的记录数量
    pub fn count(&self, table_name: &str) -> Result<usize> {
        let table = self.get_table(table_name)?;
        Ok(table.count())
    }

    /// 创建包含示例数据的数据库
    pub fn create_sample_db() -> Result<Self> {
        let key = Crypto::generate_key();
        let config = Config {
            data_dir: "./sample_data".to_string(),
            encryption_key: Some(key),
            max_file_size: 1024 * 1024,
        };

        let mut db = Self::new(config)?;

        // 创建用户表
        db.create_table("users")?;
        
        // 插入示例数据
        let mut user1 = HashMap::new();
        user1.insert("name".to_string(), Value::String("张三".to_string()));
        user1.insert("age".to_string(), Value::Int(25));
        user1.insert("email".to_string(), Value::String("zhangsan@example.com".to_string()));
        
        let mut user2 = HashMap::new();
        user2.insert("name".to_string(), Value::String("李四".to_string()));
        user2.insert("age".to_string(), Value::Int(30));
        user2.insert("email".to_string(), Value::String("lisi@example.com".to_string()));

        db.insert("users", user1)?;
        db.insert("users", user2)?;

        // 创建产品表
        db.create_table("products")?;
        
        let mut product1 = HashMap::new();
        product1.insert("name".to_string(), Value::String("笔记本电脑".to_string()));
        product1.insert("price".to_string(), Value::Float(5999.99));
        product1.insert("category".to_string(), Value::String("电子产品".to_string()));
        
        db.insert("products", product1)?;

        db.save_all()?;
        Ok(db)
    }
}

impl Drop for SimpleDB {
    fn drop(&mut self) {
        let _ = self.save_all();
    }
} 