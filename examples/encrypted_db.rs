use simpledb::{Config, SimpleDB, Value};
use simpledb::crypto::Crypto;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SimpleDB 加密数据库示例 ===\n");

    // 生成加密密钥
    let encryption_key = Crypto::generate_key();
    println!("生成的加密密钥: {}", hex::encode(&encryption_key));
    
    // 创建带加密的数据库配置
    let config = Config {
        data_dir: "./encrypted_data".to_string(),
        encryption_key: Some(encryption_key.clone()),
        max_file_size: 1024 * 1024,
    };

    // 创建数据库实例
    let mut db = SimpleDB::new(config)?;
    
    println!("\n1. 插入敏感数据");
    
    // 插入敏感用户数据
    let mut sensitive_user = HashMap::new();
    sensitive_user.insert("name".to_string(), Value::String("机密用户".to_string()));
    sensitive_user.insert("ssn".to_string(), Value::String("123-45-6789".to_string()));
    sensitive_user.insert("credit_card".to_string(), Value::String("4111-1111-1111-1111".to_string()));
    sensitive_user.insert("salary".to_string(), Value::Int(100000));
    sensitive_user.insert("security_level".to_string(), Value::String("TOP_SECRET".to_string()));
    
    let user_id = db.insert("sensitive_users", sensitive_user)?;
    println!("  插入敏感用户数据，ID: {}", user_id);
    
    // 插入银行账户信息
    let mut account = HashMap::new();
    account.insert("account_number".to_string(), Value::String("1234567890".to_string()));
    account.insert("balance".to_string(), Value::Float(50000.75));
    account.insert("owner_id".to_string(), Value::String(user_id.clone()));
    account.insert("bank_name".to_string(), Value::String("安全银行".to_string()));
    
    let account_id = db.insert("bank_accounts", account)?;
    println!("  插入银行账户信息，ID: {}", account_id);
    
    println!("\n2. 查询加密数据");
    
    // 查询敏感用户
    if let Some(user) = db.find_by_id("sensitive_users", &user_id)? {
        println!("  查询到敏感用户:");
        for (key, value) in &user.data {
            println!("    {}: {:?}", key, value);
        }
    }
    
    // 查询银行账户
    let accounts = db.find_all("bank_accounts")?;
    println!("\n  银行账户数量: {}", accounts.len());
    for account in accounts {
        println!("    账户ID: {}", account.id);
        if let Some(Value::String(account_num)) = account.data.get("account_number") {
            println!("    账户号: {}", account_num);
        }
        if let Some(Value::Float(balance)) = account.data.get("balance") {
            println!("    余额: ¥{:.2}", balance);
        }
    }
    
    println!("\n3. 保存加密数据");
    db.save_all()?;
    println!("  所有敏感数据已加密保存到磁盘");
    
    println!("\n4. 验证数据持久化");
    
    // 创建新的数据库实例来验证数据持久化
    let config2 = Config {
        data_dir: "./encrypted_data".to_string(),
        encryption_key: Some(encryption_key.clone()),
        max_file_size: 1024 * 1024,
    };
    
    let db2 = SimpleDB::new(config2)?;
    
    // 验证数据是否正确加载
    let loaded_users = db2.find_all("sensitive_users")?;
    println!("  从磁盘加载的敏感用户数量: {}", loaded_users.len());
    
    for user in loaded_users {
        if let Some(Value::String(name)) = user.data.get("name") {
            println!("    用户: {}", name);
        }
        if let Some(Value::String(security_level)) = user.data.get("security_level") {
            println!("    安全级别: {}", security_level);
        }
    }
    
    println!("\n5. 错误密钥测试");
    
    // 尝试使用错误的密钥
    let wrong_key = Crypto::generate_key(); // 生成不同的密钥
    let wrong_config = Config {
        data_dir: "./encrypted_data".to_string(),
        encryption_key: Some(wrong_key),
        max_file_size: 1024 * 1024,
    };
    
    match SimpleDB::new(wrong_config) {
        Ok(_) => {
            // 尝试访问数据，应该失败
            println!("  警告: 使用错误密钥但未检测到错误");
        }
        Err(e) => {
            println!("  正确: 使用错误密钥访问数据失败: {}", e);
        }
    }
    
    println!("\n=== 加密示例完成 ===");
    println!("注意事项:");
    println!("- 所有数据都使用AES-256-GCM加密存储");
    println!("- 没有正确的密钥无法访问数据");
    println!("- 请妥善保管您的加密密钥");
    println!("- 加密数据文件保存在 './encrypted_data' 目录中");
    
    Ok(())
} 