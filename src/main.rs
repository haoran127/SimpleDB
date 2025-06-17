use clap::{Parser, Subcommand};
use simpledb::{Config, SimpleDB, Value};
use simpledb::api::DatabaseServer;
use simpledb::crypto::Crypto;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "simpledb")]
#[command(about = "一个简单的Rust数据库")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动数据库服务器
    Server {
        #[arg(short, long, default_value = "8080")]
        port: u16,
        
        #[arg(short, long, default_value = "./data")]
        data_dir: String,
        
        #[arg(short, long)]
        encrypted: bool,
    },
    /// 创建示例数据库
    Demo {
        #[arg(short, long, default_value = "./demo_data")]
        data_dir: String,
    },
    /// 数据库操作
    Db {
        #[command(subcommand)]
        operation: DbOperation,
    },
}

#[derive(Subcommand)]
enum DbOperation {
    /// 插入记录
    Insert {
        #[arg(short, long)]
        table: String,
        
        #[arg(short, long)]
        data: String, // JSON格式
    },
    /// 查询记录
    Find {
        #[arg(short, long)]
        table: String,
        
        #[arg(short, long)]
        id: Option<String>,
    },
    /// 更新记录
    Update {
        #[arg(short, long)]
        table: String,
        
        #[arg(short, long)]
        id: String,
        
        #[arg(short, long)]
        data: String, // JSON格式
    },
    /// 删除记录
    Delete {
        #[arg(short, long)]
        table: String,
        
        #[arg(short, long)]
        id: String,
    },
    /// 列出所有表
    Tables,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port, data_dir, encrypted } => {
            println!("正在启动数据库服务器...");
            
            let config = if encrypted {
                let key = Crypto::generate_key();
                println!("生成的加密密钥（请保存）: {:?}", hex::encode(&key));
                Config {
                    data_dir,
                    encryption_key: Some(key),
                    max_file_size: 1024 * 1024 * 10,
                }
            } else {
                Config {
                    data_dir,
                    encryption_key: None,
                    max_file_size: 1024 * 1024 * 10,
                }
            };
            
            let db = SimpleDB::new(config)?;
            let server = DatabaseServer::new(db, port);
            server.start().await?;
        }
        
        Commands::Demo { data_dir } => {
            println!("正在创建示例数据库...");
            
            let config = Config {
                data_dir,
                encryption_key: Some(Crypto::generate_key()),
                max_file_size: 1024 * 1024,
            };
            
            let mut db = SimpleDB::new(config)?;
            create_demo_data(&mut db)?;
            
            println!("示例数据库创建完成！");
            println!("包含以下表和数据：");
            
            for table_name in db.list_tables() {
                let count = db.count(&table_name)?;
                println!("  - {}: {} 条记录", table_name, count);
                
                // 显示前3条记录
                let records = db.find_all(&table_name)?;
                for (i, record) in records.iter().take(3).enumerate() {
                    println!("    记录 {}: ID={}", i + 1, record.id);
                    for (key, value) in &record.data {
                        println!("      {}: {:?}", key, value);
                    }
                }
                if records.len() > 3 {
                    println!("    ... 还有 {} 条记录", records.len() - 3);
                }
            }
        }
        
        Commands::Db { operation } => {
            let config = Config::default();
            let mut db = SimpleDB::new(config)?;
            
            match operation {
                DbOperation::Insert { table, data } => {
                    let json_data: HashMap<String, serde_json::Value> = serde_json::from_str(&data)?;
                    let converted_data = convert_json_to_value(json_data);
                    let id = db.insert(&table, converted_data)?;
                    println!("记录插入成功，ID: {}", id);
                }
                
                DbOperation::Find { table, id } => {
                    if let Some(id) = id {
                        if let Some(record) = db.find_by_id(&table, &id)? {
                            print_record(record);
                        } else {
                            println!("记录不存在");
                        }
                    } else {
                        let records = db.find_all(&table)?;
                        println!("找到 {} 条记录:", records.len());
                        for record in records {
                            print_record(record);
                            println!("---");
                        }
                    }
                }
                
                DbOperation::Update { table, id, data } => {
                    let json_data: HashMap<String, serde_json::Value> = serde_json::from_str(&data)?;
                    let converted_data = convert_json_to_value(json_data);
                    db.update(&table, &id, converted_data)?;
                    println!("记录更新成功");
                }
                
                DbOperation::Delete { table, id } => {
                    db.delete(&table, &id)?;
                    println!("记录删除成功");
                }
                
                DbOperation::Tables => {
                    let tables = db.list_tables();
                    if tables.is_empty() {
                        println!("没有找到任何表");
                    } else {
                        println!("数据库中的表:");
                        for table in tables {
                            let count = db.count(&table)?;
                            println!("  - {}: {} 条记录", table, count);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn create_demo_data(db: &mut SimpleDB) -> Result<(), Box<dyn std::error::Error>> {
    // 创建用户表
    let mut user1 = HashMap::new();
    user1.insert("name".to_string(), Value::String("张三".to_string()));
    user1.insert("age".to_string(), Value::Int(25));
    user1.insert("email".to_string(), Value::String("zhangsan@example.com".to_string()));
    user1.insert("active".to_string(), Value::Bool(true));
    
    let mut user2 = HashMap::new();
    user2.insert("name".to_string(), Value::String("李四".to_string()));
    user2.insert("age".to_string(), Value::Int(30));
    user2.insert("email".to_string(), Value::String("lisi@example.com".to_string()));
    user2.insert("active".to_string(), Value::Bool(false));
    
    let mut user3 = HashMap::new();
    user3.insert("name".to_string(), Value::String("王五".to_string()));
    user3.insert("age".to_string(), Value::Int(28));
    user3.insert("email".to_string(), Value::String("wangwu@example.com".to_string()));
    user3.insert("active".to_string(), Value::Bool(true));

    db.insert("users", user1)?;
    db.insert("users", user2)?;
    db.insert("users", user3)?;

    // 创建产品表
    let mut product1 = HashMap::new();
    product1.insert("name".to_string(), Value::String("笔记本电脑".to_string()));
    product1.insert("price".to_string(), Value::Float(5999.99));
    product1.insert("category".to_string(), Value::String("电子产品".to_string()));
    product1.insert("in_stock".to_string(), Value::Bool(true));
    
    let mut product2 = HashMap::new();
    product2.insert("name".to_string(), Value::String("智能手机".to_string()));
    product2.insert("price".to_string(), Value::Float(2999.50));
    product2.insert("category".to_string(), Value::String("电子产品".to_string()));
    product2.insert("in_stock".to_string(), Value::Bool(true));
    
    let mut product3 = HashMap::new();
    product3.insert("name".to_string(), Value::String("咖啡机".to_string()));
    product3.insert("price".to_string(), Value::Float(899.00));
    product3.insert("category".to_string(), Value::String("家电".to_string()));
    product3.insert("in_stock".to_string(), Value::Bool(false));

    db.insert("products", product1)?;
    db.insert("products", product2)?;
    db.insert("products", product3)?;

    // 创建订单表
    let mut order1 = HashMap::new();
    order1.insert("user_name".to_string(), Value::String("张三".to_string()));
    order1.insert("product_name".to_string(), Value::String("笔记本电脑".to_string()));
    order1.insert("quantity".to_string(), Value::Int(1));
    order1.insert("total".to_string(), Value::Float(5999.99));
    order1.insert("status".to_string(), Value::String("已支付".to_string()));

    db.insert("orders", order1)?;

    db.save_all()?;
    Ok(())
}

fn convert_json_to_value(json_map: HashMap<String, serde_json::Value>) -> HashMap<String, Value> {
    json_map
        .into_iter()
        .map(|(k, v)| {
            let value = match v {
                serde_json::Value::Null => Value::Null,
                serde_json::Value::Bool(b) => Value::Bool(b),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Value::Int(i)
                    } else if let Some(f) = n.as_f64() {
                        Value::Float(f)
                    } else {
                        Value::Null
                    }
                }
                serde_json::Value::String(s) => Value::String(s),
                _ => Value::String(v.to_string()),
            };
            (k, value)
        })
        .collect()
}

fn print_record(record: &simpledb::storage::Record) {
    println!("记录 ID: {}", record.id);
    println!("创建时间: {}", record.created_at);
    println!("更新时间: {}", record.updated_at);
    println!("数据:");
    for (key, value) in &record.data {
        println!("  {}: {:?}", key, value);
    }
} 