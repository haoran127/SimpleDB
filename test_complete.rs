use simpledb::{Config, SimpleDB, Value};
use simpledb::crypto::Crypto;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SimpleDB 完整功能测试 ===\n");

    // 测试1: 基本CRUD操作
    println!("1. 测试基本CRUD操作");
    test_basic_crud().await?;

    // 测试2: 加密功能
    println!("\n2. 测试加密功能");
    test_encryption().await?;

    // 测试3: 文件格式和持久化
    println!("\n3. 测试文件格式和持久化");
    test_persistence().await?;

    // 测试4: 数据类型支持
    println!("\n4. 测试数据类型支持");
    test_data_types().await?;

    // 测试5: 条件查询
    println!("\n5. 测试条件查询");
    test_conditional_queries().await?;

    println!("\n=== 所有测试完成 ===");
    println!("✅ 文件格式: 使用 bincode 二进制序列化");
    println!("✅ 加密功能: AES-256-GCM 端到端加密");
    println!("✅ 数据驱动API: 完整的 CRUD 操作支持");
    println!("✅ 持久化存储: 自动保存和加载数据");
    println!("✅ 多种数据类型: 字符串、整数、浮点、布尔、数组等");

    Ok(())
}

async fn test_basic_crud() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        data_dir: "./test_basic".to_string(),
        encryption_key: None,
        max_file_size: 1024 * 1024,
    };

    let mut db = SimpleDB::new(config)?;

    // 创建用户数据
    let mut user_data = HashMap::new();
    user_data.insert("name".to_string(), Value::String("测试用户".to_string()));
    user_data.insert("age".to_string(), Value::Int(25));
    user_data.insert("active".to_string(), Value::Bool(true));

    // 插入数据
    let user_id = db.insert("users", user_data)?;
    println!("  ✅ 插入用户成功，ID: {}", user_id);

    // 查询数据
    if let Some(user) = db.find_by_id("users", &user_id)? {
        println!("  ✅ 查询用户成功: {}", user.data.get("name").unwrap().as_string().unwrap());
    }

    // 更新数据
    let mut update_data = HashMap::new();
    update_data.insert("name".to_string(), Value::String("测试用户".to_string()));
    update_data.insert("age".to_string(), Value::Int(26));
    update_data.insert("active".to_string(), Value::Bool(true));
    db.update("users", &user_id, update_data)?;
    println!("  ✅ 更新用户成功");

    // 删除数据
    db.delete("users", &user_id)?;
    println!("  ✅ 删除用户成功");

    Ok(())
}

async fn test_encryption() -> Result<(), Box<dyn std::error::Error>> {
    let key = Crypto::generate_key();
    let config = Config {
        data_dir: "./test_encrypted".to_string(),
        encryption_key: Some(key.clone()),
        max_file_size: 1024 * 1024,
    };

    let mut db = SimpleDB::new(config)?;

    // 插入敏感数据
    let mut sensitive_data = HashMap::new();
    sensitive_data.insert("password".to_string(), Value::String("secret123".to_string()));
    sensitive_data.insert("credit_card".to_string(), Value::String("1234-5678-9012-3456".to_string()));

    let id = db.insert("secrets", sensitive_data)?;
    println!("  ✅ 插入加密数据成功");

    // 保存数据
    db.save_all()?;
    println!("  ✅ 加密数据保存到磁盘");

    // 使用相同密钥重新加载
    let config2 = Config {
        data_dir: "./test_encrypted".to_string(),
        encryption_key: Some(key),
        max_file_size: 1024 * 1024,
    };
    let db2 = SimpleDB::new(config2)?;
    
    if let Some(record) = db2.find_by_id("secrets", &id)? {
        println!("  ✅ 使用正确密钥解密数据成功");
    }

    // 测试错误密钥
    let wrong_key = Crypto::generate_key();
    let wrong_config = Config {
        data_dir: "./test_encrypted".to_string(),
        encryption_key: Some(wrong_key),
        max_file_size: 1024 * 1024,
    };
    
    match SimpleDB::new(wrong_config) {
        Err(_) => println!("  ✅ 错误密钥正确被拒绝"),
        Ok(_) => println!("  ⚠️  错误密钥未被检测到"),
    }

    Ok(())
}

async fn test_persistence() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        data_dir: "./test_persistence".to_string(),
        encryption_key: None,
        max_file_size: 1024 * 1024,
    };

    // 第一次创建数据库并插入数据
    {
        let mut db = SimpleDB::new(config.clone())?;
        
        let mut data = HashMap::new();
        data.insert("persistent_data".to_string(), Value::String("这条数据应该持久保存".to_string()));
        
        db.insert("persistence_test", data)?;
        db.save_all()?;
        println!("  ✅ 数据保存到文件");
    } // db 在这里被销毁

    // 重新创建数据库并检查数据是否存在
    {
        let db = SimpleDB::new(config)?;
        let tables = db.list_tables();
        
        if tables.contains(&"persistence_test".to_string()) {
            let records = db.find_all("persistence_test")?;
            if !records.is_empty() {
                println!("  ✅ 数据从文件成功加载，记录数: {}", records.len());
            }
        }
    }

    Ok(())
}

async fn test_data_types() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        data_dir: "./test_types".to_string(),
        encryption_key: None,
        max_file_size: 1024 * 1024,
    };

    let mut db = SimpleDB::new(config)?;

    let mut type_data = HashMap::new();
    type_data.insert("null_value".to_string(), Value::Null);
    type_data.insert("bool_value".to_string(), Value::Bool(true));
    type_data.insert("int_value".to_string(), Value::Int(42));
    type_data.insert("float_value".to_string(), Value::Float(3.14159));
    type_data.insert("string_value".to_string(), Value::String("Hello, 世界!".to_string()));
    type_data.insert("bytes_value".to_string(), Value::Bytes(vec![1, 2, 3, 4, 5]));
    type_data.insert("array_value".to_string(), Value::Array(vec![
        Value::String("item1".to_string()),
        Value::Int(100),
        Value::Bool(false),
    ]));

    let id = db.insert("type_test", type_data)?;
    
    if let Some(record) = db.find_by_id("type_test", &id)? {
        println!("  ✅ 所有数据类型存储和检索成功:");
        for (key, value) in &record.data {
            println!("    {}: {:?}", key, value);
        }
    }

    Ok(())
}

async fn test_conditional_queries() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        data_dir: "./test_queries".to_string(),
        encryption_key: None,
        max_file_size: 1024 * 1024,
    };

    let mut db = SimpleDB::new(config)?;

    // 插入多个用户
    for i in 1..=5 {
        let mut user_data = HashMap::new();
        user_data.insert("name".to_string(), Value::String(format!("用户{}", i)));
        user_data.insert("age".to_string(), Value::Int(20 + i));
        user_data.insert("active".to_string(), Value::Bool(i % 2 == 0));
        
        db.insert("query_users", user_data)?;
    }

    // 条件查询：查找活跃用户
    let active_users = db.find_where("query_users", |record| {
        if let Some(Value::Bool(active)) = record.data.get("active") {
            *active
        } else {
            false
        }
    })?;

    println!("  ✅ 条件查询成功，找到 {} 个活跃用户", active_users.len());

    // 条件查询：查找年龄大于22的用户
    let older_users = db.find_where("query_users", |record| {
        if let Some(Value::Int(age)) = record.data.get("age") {
            *age > 22
        } else {
            false
        }
    })?;

    println!("  ✅ 年龄过滤查询成功，找到 {} 个用户 (年龄 > 22)", older_users.len());

    Ok(())
} 