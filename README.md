# SimpleDB - Rust简单数据库

一个用Rust实现的简单数据库，支持文件存储、AES加密和RESTful API。

## 特性

- 🗄️ **文件存储格式**: 使用bincode进行高效的二进制序列化
- 🔐 **AES-GCM加密**: 可选的端到端数据加密保护
- 🚀 **高性能API**: 异步HTTP服务器提供RESTful接口
- 📊 **灵活数据类型**: 支持多种数据类型（字符串、整数、浮点数、布尔值、数组等）
- 🔍 **查询功能**: 支持按ID查询和条件过滤
- 💾 **持久化存储**: 自动数据持久化到磁盘
- 🛠️ **命令行工具**: 完整的CLI界面进行数据库操作

## 项目架构

```
src/
├── lib.rs          # 主库入口和配置
├── main.rs         # 命令行工具
├── error.rs        # 错误处理
├── crypto.rs       # AES加密模块
├── storage.rs      # 存储和文件格式
├── database.rs     # 数据库主类
└── api.rs          # HTTP API服务器
```

### 核心组件

1. **存储层 (Storage)**
   - `Record`: 数据记录结构
   - `Value`: 支持的数据类型枚举
   - `Table`: 表管理和文件操作

2. **加密层 (Crypto)**
   - AES-256-GCM加密算法
   - 自动nonce生成
   - 安全密钥管理

3. **数据库层 (Database)**
   - `SimpleDB`: 主数据库类
   - 表管理和CRUD操作
   - 自动持久化

4. **API层 (API)**
   - HTTP REST服务器
   - JSON序列化/反序列化
   - 异步请求处理

## 安装和构建

```bash
# 克隆项目
git clone <repository>
cd simpledb

# 构建项目
cargo build --release

# 运行测试
cargo test
```

## 使用方法

### 1. 命令行工具

#### 创建示例数据库
```bash
cargo run demo --data-dir ./demo_data
```

#### 启动数据库服务器
```bash
# 无加密
cargo run server --port 8080 --data-dir ./data

# 启用加密
cargo run server --port 8080 --data-dir ./data --encrypted
```

#### 数据库操作
```bash
# 插入记录
cargo run db insert --table users --data '{"name":"张三","age":25,"email":"zhangsan@example.com"}'

# 查询所有记录
cargo run db find --table users

# 根据ID查询
cargo run db find --table users --id <record_id>

# 更新记录
cargo run db update --table users --id <record_id> --data '{"name":"张三","age":26}'

# 删除记录
cargo run db delete --table users --id <record_id>

# 列出所有表
cargo run db tables
```

### 2. HTTP API

启动服务器后，可以通过HTTP API进行操作：

#### 插入记录
```bash
curl -X POST http://localhost:8080/api/insert \
  -H "Content-Type: application/json" \
  -d '{
    "table": "users",
    "data": {
      "name": "李四",
      "age": 30,
      "email": "lisi@example.com"
    }
  }'
```

#### 查询记录
```bash
# 查询所有记录
curl -X GET http://localhost:8080/api/find \
  -H "Content-Type: application/json" \
  -d '{"table": "users"}'

# 根据ID查询
curl -X GET http://localhost:8080/api/find \
  -H "Content-Type: application/json" \
  -d '{"table": "users", "id": "<record_id>"}'
```

#### 更新记录
```bash
curl -X PUT http://localhost:8080/api/update \
  -H "Content-Type: application/json" \
  -d '{
    "table": "users",
    "id": "<record_id>",
    "data": {
      "name": "李四",
      "age": 31
    }
  }'
```

#### 删除记录
```bash
curl -X DELETE http://localhost:8080/api/delete \
  -H "Content-Type: application/json" \
  -d '{
    "table": "users",
    "id": "<record_id>"
  }'
```

#### 列出所有表
```bash
curl -X GET http://localhost:8080/api/tables
```

### 3. 编程接口

```rust
use simpledb::{Config, SimpleDB, Value};
use std::collections::HashMap;

// 创建数据库
let config = Config::default();
let mut db = SimpleDB::new(config)?;

// 插入数据
let mut data = HashMap::new();
data.insert("name".to_string(), Value::String("张三".to_string()));
data.insert("age".to_string(), Value::Int(25));

let id = db.insert("users", data)?;

// 查询数据
let record = db.find_by_id("users", &id)?;

// 更新数据
let mut update_data = HashMap::new();
update_data.insert("age".to_string(), Value::Int(26));
db.update("users", &id, update_data)?;

// 删除数据
db.delete("users", &id)?;
```

## 文件格式

数据以二进制格式存储在`.db`文件中：

1. **未加密**: 直接使用bincode序列化的HashMap
2. **加密**: 12字节nonce + AES-GCM加密的数据

文件结构：
```
data/
├── users.db      # 用户表数据
├── products.db   # 产品表数据
└── orders.db     # 订单表数据
```

## 支持的数据类型

- `Null`: 空值
- `Bool`: 布尔值
- `Int`: 64位整数
- `Float`: 64位浮点数
- `String`: UTF-8字符串
- `Bytes`: 字节数组
- `Array`: 值数组
- `Object`: 嵌套对象

## 加密

- **算法**: AES-256-GCM
- **密钥长度**: 256位 (32字节)
- **认证**: 内置认证标签防止篡改
- **Nonce**: 每次加密自动生成唯一nonce

## 性能特点

- **内存优化**: 采用懒加载，只在需要时加载表
- **异步IO**: 使用Tokio进行高性能异步操作
- **批量操作**: 支持批量插入和查询
- **自动持久化**: 在对象销毁时自动保存更改

## 限制

- 单表最大文件大小: 10MB (可配置)
- 内存中存储: 所有数据加载到内存中
- 无事务支持: 不支持ACID事务
- 无并发控制: 使用互斥锁进行简单的并发控制

## 示例数据

运行 `cargo run demo` 会创建包含以下数据的示例数据库：

- **users表**: 3个用户记录
- **products表**: 3个产品记录  
- **orders表**: 1个订单记录

## 开发

```bash
# 运行测试
cargo test

# 检查代码风格
cargo clippy

# 格式化代码
cargo fmt

# 生成文档
cargo doc --open
```

## 许可证

MIT License 
 