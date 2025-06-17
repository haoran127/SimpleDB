use simpledb::{Config, SimpleDB};
use simpledb::api::DatabaseServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SimpleDB API 服务器示例 ===\n");

    // 创建数据库配置（不使用加密以便测试）
    let config = Config {
        data_dir: "./api_data".to_string(),
        encryption_key: None,
        max_file_size: 1024 * 1024,
    };

    // 创建数据库实例
    let db = SimpleDB::new(config)?;
    
    // 创建API服务器
    let server = DatabaseServer::new(db, 8080);
    
    println!("正在启动数据库API服务器...");
    println!("服务器地址: http://localhost:8080");
    println!("\n可用的API端点:");
    println!("  POST /api/insert   - 插入记录");
    println!("  GET  /api/find     - 查询记录");
    println!("  PUT  /api/update   - 更新记录");
    println!("  DELETE /api/delete - 删除记录");
    println!("  GET  /api/tables   - 列出所有表");
    
    println!("\n示例请求:");
    println!("# 插入用户");
    println!("curl -X POST http://localhost:8080/api/insert \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{");
    println!("    \"table\": \"users\",");
    println!("    \"data\": {{");
    println!("      \"name\": \"张三\",");
    println!("      \"age\": 25,");
    println!("      \"email\": \"zhangsan@example.com\"");
    println!("    }}");
    println!("  }}'");
    
    println!("\n# 查询所有用户");
    println!("curl -X GET http://localhost:8080/api/find \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{\"table\": \"users\"}}'");
    
    println!("\n按 Ctrl+C 停止服务器\n");
    
    // 启动服务器（这会阻塞直到程序被中断）
    server.start().await?;
    
    Ok(())
} 