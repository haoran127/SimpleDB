use simpledb::{Config, SimpleDB, Value};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SimpleDB 基本使用示例 ===\n");

    // 创建数据库配置
    let config = Config {
        data_dir: "./example_data".to_string(),
        encryption_key: None, // 不使用加密
        max_file_size: 1024 * 1024,
    };

    // 创建数据库实例
    let mut db = SimpleDB::new(config)?;
    
    println!("1. 插入用户数据");
    
    // 插入第一个用户
    let mut user1 = HashMap::new();
    user1.insert("name".to_string(), Value::String("张三".to_string()));
    user1.insert("age".to_string(), Value::Int(25));
    user1.insert("email".to_string(), Value::String("zhangsan@example.com".to_string()));
    user1.insert("active".to_string(), Value::Bool(true));
    
    let user1_id = db.insert("users", user1)?;
    println!("  插入用户1，ID: {}", user1_id);
    
    // 插入第二个用户
    let mut user2 = HashMap::new();
    user2.insert("name".to_string(), Value::String("李四".to_string()));
    user2.insert("age".to_string(), Value::Int(30));
    user2.insert("email".to_string(), Value::String("lisi@example.com".to_string()));
    user2.insert("active".to_string(), Value::Bool(false));
    
    let user2_id = db.insert("users", user2)?;
    println!("  插入用户2，ID: {}", user2_id);
    
    println!("\n2. 查询数据");
    
    // 查询所有用户
    let all_users = db.find_all("users")?;
    println!("  总共有 {} 个用户:", all_users.len());
    for user in &all_users {
        println!("    ID: {}", user.id);
        if let Some(name) = user.data.get("name") {
            println!("    姓名: {:?}", name);
        }
        if let Some(age) = user.data.get("age") {
            println!("    年龄: {:?}", age);
        }
        println!("    ---");
    }
    
    // 根据ID查询特定用户
    if let Some(user) = db.find_by_id("users", &user1_id)? {
        println!("  查询用户1 (ID: {}):", user.id);
        for (key, value) in &user.data {
            println!("    {}: {:?}", key, value);
        }
    }
    
    println!("\n3. 更新数据");
    
    // 更新用户1的年龄
    let mut update_data = HashMap::new();
    update_data.insert("age".to_string(), Value::Int(26));
    update_data.insert("name".to_string(), Value::String("张三".to_string()));
    update_data.insert("email".to_string(), Value::String("zhangsan@example.com".to_string()));
    update_data.insert("active".to_string(), Value::Bool(true));
    update_data.insert("last_login".to_string(), Value::String("2024-01-01".to_string()));
    
    db.update("users", &user1_id, update_data)?;
    println!("  更新用户1的数据");
    
    // 验证更新
    if let Some(updated_user) = db.find_by_id("users", &user1_id)? {
        println!("  更新后的用户1数据:");
        for (key, value) in &updated_user.data {
            println!("    {}: {:?}", key, value);
        }
    }
    
    println!("\n4. 条件查询");
    
    // 查询活跃用户
    let active_users = db.find_where("users", |record| {
        if let Some(Value::Bool(active)) = record.data.get("active") {
            *active
        } else {
            false
        }
    })?;
    
    println!("  活跃用户数量: {}", active_users.len());
    for user in active_users {
        if let Some(Value::String(name)) = user.data.get("name") {
            println!("    活跃用户: {}", name);
        }
    }
    
    println!("\n5. 插入产品数据");
    
    let mut product1 = HashMap::new();
    product1.insert("name".to_string(), Value::String("笔记本电脑".to_string()));
    product1.insert("price".to_string(), Value::Float(5999.99));
    product1.insert("category".to_string(), Value::String("电子产品".to_string()));
    product1.insert("in_stock".to_string(), Value::Bool(true));
    
    let product1_id = db.insert("products", product1)?;
    println!("  插入产品，ID: {}", product1_id);
    
    println!("\n6. 列出所有表");
    
    let tables = db.list_tables();
    for table_name in tables {
        let count = db.count(&table_name)?;
        println!("  表 '{}': {} 条记录", table_name, count);
    }
    
    println!("\n7. 删除数据");
    
    // 删除用户2
    db.delete("users", &user2_id)?;
    println!("  删除用户2 (ID: {})", user2_id);
    
    // 验证删除
    let remaining_users = db.find_all("users")?;
    println!("  删除后剩余用户数量: {}", remaining_users.len());
    
    println!("\n8. 保存数据到磁盘");
    db.save_all()?;
    println!("  所有数据已保存到磁盘");
    
    println!("\n=== 示例完成 ===");
    println!("数据文件保存在 './example_data' 目录中");
    
    Ok(())
} 