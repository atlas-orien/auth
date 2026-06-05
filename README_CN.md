# auth

**中文** | [English](./README.md)

一个用 Rust 编写的轻量级认证服务，提供用户注册、登录、JWT Token 管理以及基本用户信息操作的 REST API。

## 功能特性

- 用户注册与登录（支持用户名或邮箱）
- JWT 访问令牌 + 刷新令牌对
- Token 刷新接口
- 获取当前用户信息（`/me`）
- 修改密码
- 修改邮箱
- 邮箱验证

## 快速开始

### 1. 准备环境

```bash
# 启动 PostgreSQL（需要 Docker）
make postgres

# 复制并编辑配置文件
cp config/services-example.toml config/services.toml

# 生成 Ed25519 密钥对（access + refresh）
bash scripts/gen_jwt_keys.sh
```

根据实际情况修改 `config/services.toml` 中的数据库地址和 JWT issuer/audience。

### 2. 执行数据库迁移

```bash
make migrate-up
```

### 3. 启动服务

```bash
cargo run -p auth-server
```

## API 接口

### 会话 (`/auth/session`)

| 方法   | 路径             | 描述                    | 是否需要认证 |
| ------ | ---------------- | ----------------------- | ------------ |
| POST   | `/register`      | 注册新用户              | 否           |
| POST   | `/login`         | 登录，返回令牌对        | 否           |
| POST   | `/refresh_token` | 刷新访问令牌            | 否           |

### 用户 (`/auth/user`)

| 方法   | 路径             | 描述             | 是否需要认证      |
| ------ | ---------------- | ---------------- | ----------------- |
| GET    | `/me`            | 获取当前用户信息 | 是（Bearer Token）|
| PUT    | `/password`      | 修改密码         | 是（Bearer Token）|
| PUT    | `/email`         | 修改邮箱         | 是（Bearer Token）|
| PUT    | `/profile`       | 修改显示名和头像 | 是（Bearer Token）|
| POST   | `/email/verify`  | 验证邮箱         | 是（Bearer Token）|

---

## 架构

```text
┌──────────────────────────────────────────────────────┐
│                    web-server                        │  HTTP API（Axum）
├──────────────────────────────────────────────────────┤
│                     service                          │  业务逻辑层
├──────────────────────────────────────────────────────┤
│                      repo                            │  数据访问层（SeaORM）
├──────────────────────────────────────────────────────┤
│               （外部库）db-core-rs                   │  基础设施与共享核心
├──────────────────────────────────────────────────────┤
│                    migration                         │  数据库迁移（SeaORM）
└──────────────────────────────────────────────────────┘
```

## 数据库结构

**users** 表：

| 字段           | 类型        | 说明             |
| -------------- | ----------- | ---------------- |
| id             | string (PK) | 用户唯一 ID      |
| display_user_id| string      | 对外展示 ID，可空且唯一 |
| username       | string      | 唯一             |
| display_name   | string      | 可为空           |
| avatar         | string      | 可为空（URL）    |
| password       | string      | 哈希存储         |
| email          | string      | 可为空，唯一     |
| email_verified | boolean     | 默认 false       |
| disabled       | boolean     | 默认 false       |
| created_at     | timestamptz |                  |
| updated_at     | timestamptz |                  |

## 配置文件（`config/services.toml`）

```toml
[http]
port = 19878

[jwt]
issuer = "auth-server"
audience = "test"
key_dir = "config/key"
access_token_duration = 10800   # 3 小时
refresh_token_duration = 604800 # 1 周

[[db]]
name = "default"
url = "postgres://postgres:123456@localhost:15432/auth"
```

## 技术栈

| 组件      | 技术                |
| --------- | ------------------- |
| 运行时    | Tokio               |
| ORM       | SeaORM              |
| Web 框架  | Axum 0.8            |
| JWT       | toolcraft-jwt       |
| 核心库    | db-core-rs          |

## 开发命令

```bash
make help           # 查看所有命令
make postgres       # 启动 PostgreSQL 容器
make migrate-up     # 执行待处理的迁移
make migrate-fresh  # 重置数据库并重新执行所有迁移
make build          # 编译所有 crate
```

## License

MIT or Apache-2.0
