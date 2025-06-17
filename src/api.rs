use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use base64::Engine;

use crate::database::SimpleDB;
use crate::error::{DatabaseError, Result};
use crate::storage::Value;

/// HTTP请求结构
#[derive(Debug, Deserialize)]
pub struct ApiRequest {
    pub method: String,
    pub table: String,
    pub id: Option<String>,
    pub data: Option<HashMap<String, serde_json::Value>>,
    pub query: Option<HashMap<String, serde_json::Value>>,
}

/// HTTP响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub message: Option<String>,
}

impl ApiResponse {
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            message: None,
        }
    }

    pub fn message(msg: String) -> Self {
        Self {
            success: true,
            data: None,
            error: None,
            message: Some(msg),
        }
    }
}

/// 数据库API服务器
pub struct DatabaseServer {
    db: Arc<Mutex<SimpleDB>>,
    port: u16,
}

impl DatabaseServer {
    pub fn new(db: SimpleDB, port: u16) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            port,
        }
    }

    /// 启动服务器
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .await
            .map_err(|e| DatabaseError::Io(e))?;

        println!("数据库服务器启动，监听端口: {}", self.port);
        println!("API文档:");
        println!("  POST /api/insert   - 插入记录");
        println!("  GET  /api/find     - 查询记录");
        println!("  PUT  /api/update   - 更新记录");
        println!("  DELETE /api/delete - 删除记录");
        println!("  GET  /api/tables   - 列出所有表");

        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let db = Arc::clone(&self.db);
                    tokio::spawn(async move {
                        let mut buffer = [0; 1024];
                        
                        match stream.read(&mut buffer).await {
                            Ok(n) if n > 0 => {
                                let request = String::from_utf8_lossy(&buffer[..n]);
                                let response = Self::handle_request(&db, &request).await;
                                let response_json = serde_json::to_string(&response).unwrap();
                                
                                let http_response = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                                    response_json.len(),
                                    response_json
                                );
                                
                                let _ = stream.write_all(http_response.as_bytes()).await;
                            }
                            _ => {}
                        }
                    });
                }
                Err(_) => continue,
            }
        }
    }

    /// 处理HTTP请求
    async fn handle_request(db: &Arc<Mutex<SimpleDB>>, request: &str) -> ApiResponse {
        // 简单的HTTP请求解析
        let lines: Vec<&str> = request.lines().collect();
        if lines.is_empty() {
            return ApiResponse::error("无效的请求".to_string());
        }

        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return ApiResponse::error("无效的请求格式".to_string());
        }

        let method = parts[0];
        let path = parts[1];

        // 查找请求体
        let mut body = "";
        for (i, line) in lines.iter().enumerate() {
            if line.is_empty() && i + 1 < lines.len() {
                body = lines[i + 1];
                break;
            }
        }

        // 路由处理
        match (method, path) {
            ("POST", "/api/insert") => Self::handle_insert(db, body).await,
            ("GET", "/api/find") => Self::handle_find(db, body).await,
            ("PUT", "/api/update") => Self::handle_update(db, body).await,
            ("DELETE", "/api/delete") => Self::handle_delete(db, body).await,
            ("GET", "/api/tables") => Self::handle_list_tables(db).await,
            _ => ApiResponse::error("不支持的API端点".to_string()),
        }
    }

    /// 处理插入请求
    async fn handle_insert(db: &Arc<Mutex<SimpleDB>>, body: &str) -> ApiResponse {
        match serde_json::from_str::<ApiRequest>(body) {
            Ok(req) => {
                if let Some(data) = req.data {
                    let converted_data = Self::convert_json_to_value(data);
                    match db.lock().unwrap().insert(&req.table, converted_data) {
                        Ok(id) => ApiResponse::success(serde_json::json!({"id": id})),
                        Err(e) => ApiResponse::error(format!("插入失败: {}", e)),
                    }
                } else {
                    ApiResponse::error("缺少数据字段".to_string())
                }
            }
            Err(e) => ApiResponse::error(format!("JSON解析错误: {}", e)),
        }
    }

    /// 处理查询请求
    async fn handle_find(db: &Arc<Mutex<SimpleDB>>, body: &str) -> ApiResponse {
        match serde_json::from_str::<ApiRequest>(body) {
            Ok(req) => {
                let db_guard = db.lock().unwrap();
                if let Some(id) = req.id {
                    // 根据ID查询
                    match db_guard.find_by_id(&req.table, &id) {
                        Ok(Some(record)) => {
                            let json_record = Self::convert_record_to_json(record);
                            ApiResponse::success(json_record)
                        }
                        Ok(None) => ApiResponse::error("记录不存在".to_string()),
                        Err(e) => ApiResponse::error(format!("查询失败: {}", e)),
                    }
                } else {
                    // 查询所有记录
                    match db_guard.find_all(&req.table) {
                        Ok(records) => {
                            let json_records: Vec<_> = records
                                .iter()
                                .map(|r| Self::convert_record_to_json(r))
                                .collect();
                            ApiResponse::success(serde_json::json!(json_records))
                        }
                        Err(e) => ApiResponse::error(format!("查询失败: {}", e)),
                    }
                }
            }
            Err(e) => ApiResponse::error(format!("JSON解析错误: {}", e)),
        }
    }

    /// 处理更新请求
    async fn handle_update(db: &Arc<Mutex<SimpleDB>>, body: &str) -> ApiResponse {
        match serde_json::from_str::<ApiRequest>(body) {
            Ok(req) => {
                if let (Some(id), Some(data)) = (req.id, req.data) {
                    let converted_data = Self::convert_json_to_value(data);
                    match db.lock().unwrap().update(&req.table, &id, converted_data) {
                        Ok(_) => ApiResponse::message("更新成功".to_string()),
                        Err(e) => ApiResponse::error(format!("更新失败: {}", e)),
                    }
                } else {
                    ApiResponse::error("缺少ID或数据字段".to_string())
                }
            }
            Err(e) => ApiResponse::error(format!("JSON解析错误: {}", e)),
        }
    }

    /// 处理删除请求
    async fn handle_delete(db: &Arc<Mutex<SimpleDB>>, body: &str) -> ApiResponse {
        match serde_json::from_str::<ApiRequest>(body) {
            Ok(req) => {
                if let Some(id) = req.id {
                    match db.lock().unwrap().delete(&req.table, &id) {
                        Ok(_) => ApiResponse::message("删除成功".to_string()),
                        Err(e) => ApiResponse::error(format!("删除失败: {}", e)),
                    }
                } else {
                    ApiResponse::error("缺少ID字段".to_string())
                }
            }
            Err(e) => ApiResponse::error(format!("JSON解析错误: {}", e)),
        }
    }

    /// 处理列出表请求
    async fn handle_list_tables(db: &Arc<Mutex<SimpleDB>>) -> ApiResponse {
        let tables = db.lock().unwrap().list_tables();
        ApiResponse::success(serde_json::json!(tables))
    }

    /// 将JSON值转换为内部Value类型
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
                    serde_json::Value::Array(arr) => {
                        Value::Array(arr.into_iter().map(|v| Value::String(v.to_string())).collect())
                    }
                    serde_json::Value::Object(_) => Value::String(v.to_string()),
                };
                (k, value)
            })
            .collect()
    }

    /// 将记录转换为JSON
    fn convert_record_to_json(record: &crate::storage::Record) -> serde_json::Value {
        let mut json_map = serde_json::Map::new();
        json_map.insert("id".to_string(), serde_json::Value::String(record.id.clone()));
        json_map.insert("created_at".to_string(), serde_json::Value::Number(record.created_at.into()));
        json_map.insert("updated_at".to_string(), serde_json::Value::Number(record.updated_at.into()));

        let mut data_map = serde_json::Map::new();
        for (key, value) in &record.data {
            let json_value = match value {
                Value::Null => serde_json::Value::Null,
                Value::Bool(b) => serde_json::Value::Bool(*b),
                Value::Int(i) => serde_json::Value::Number((*i).into()),
                Value::Float(f) => serde_json::Value::Number(
                    serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0)),
                ),
                Value::String(s) => serde_json::Value::String(s.clone()),
                Value::Bytes(b) => serde_json::Value::String(base64::engine::general_purpose::STANDARD.encode(b)),
                Value::Array(arr) => serde_json::Value::Array(
                    arr.iter()
                        .map(|v| serde_json::Value::String(format!("{:?}", v)))
                        .collect(),
                ),
                Value::Object(_) => serde_json::Value::String(format!("{:?}", value)),
            };
            data_map.insert(key.clone(), json_value);
        }
        json_map.insert("data".to_string(), serde_json::Value::Object(data_map));

        serde_json::Value::Object(json_map)
    }
} 