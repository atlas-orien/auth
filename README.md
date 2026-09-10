# Rust Auth Service Template

这是一个面向 AI 协作开发的 Rust 后端模板。它已经内置认证服务的基本形态，但更重要的是提供了一套清晰的分层架构和 AI 执行协议，方便在这个模板上持续添加表、业务 API 和 HTTP 接口。

## 这个模板适合做什么

- 快速启动一个 Axum + SeaORM + PostgreSQL 的业务服务
- 用固定分层约束 AI 写代码，避免跨层乱改
- 统一处理业务错误码、参数校验、HTTP 响应
- 在已有架构上逐步扩展业务模块

当前示例业务是 auth：

- 用户注册、登录
- JWT access token / refresh token
- 用户资料、邮箱、密码相关接口
- 统一 `CommonResponse` / `CommonError` 响应结构

## 架构

```text
┌──────────────────────────────────────────────────────┐
│                    web-server                        │  Layer 5: HTTP API
├──────────────────────────────────────────────────────┤
│                     service                          │  Layer 4: Business API
├──────────────────────────────────────────────────────┤
│                      repo                            │  Layer 3: Table Services
├──────────────────────────────────────────────────────┤
│                    db_core                           │  Layer 2: Infrastructure
├──────────────────────────────────────────────────────┤
│                    migration                         │  Layer 1: Schema
└──────────────────────────────────────────────────────┘
```

核心边界：

- `migration`：只定义数据库结构
- `repo`：只做单表数据访问，是唯一允许引入 `sea-orm` 的业务 crate
- `service`：业务编排、跨表组合、业务错误
- `web-server`：HTTP DTO、validator、路由、响应转换
- `error-code`：集中维护稳定错误码

## 快速开始

### 1. 准备配置

```bash
cp config/services-example.toml config/services.toml
bash scripts/gen_jwt_keys.sh
```

按实际环境修改 `config/services.toml` 中的数据库、JWT、头像存储等配置。

### 2. 启动数据库

```bash
make postgres
```

### 3. 执行迁移并生成 entity

```bash
make fresh-db
```

这个命令会刷新 migration，并根据数据库结构重新生成 SeaORM entity。

### 4. 启动服务

```bash
cargo run -p auth-server
```

默认端口在 `config/services.toml` 的 `[http]` 中配置。

## 常用命令

```bash
make help
make build
make test
make migrate-up
make migrate-down
make migrate-fresh
make migrate-gen NAME=create_xxx
make fresh-db
```

## 如何把这个仓库当模板使用

推荐流程：

1. 复制这个仓库作为新项目基础
2. 修改 workspace、crate 名称和配置文件
3. 保留 `ai_protocols/`，这是给 AI 的执行规约
4. 先让 AI 阅读相关协议，再让 AI 修改代码
5. 每次功能开发都按 migration -> repo -> service -> web-server 顺序推进

推荐给 AI 的开场提示：

```text
请先阅读 ai_protocols/TABLE_ADDING_PROTOCOL.md，并严格按照其中的分层边界实现。
不要修改 db_core。
不要在 repo 之外引入 sea-orm。
不要手写 entity，entity 由 fresh_db.sh 生成。
```

## 新增完整功能的 AI 提示模板

```text
请严格按照 ai_protocols/TABLE_ADDING_PROTOCOL.md 执行。

需要实现完整功能：

### 1. 新增表
- 表名：<table_name>
- 字段：
  - <field1>: <type>
  - <field2>: <type>

### 2. 业务 API
- 功能：<描述>
- 输入：<参数>
- 输出：<返回>

### 3. HTTP 接口
- <METHOD> /<path> - <描述>

按 5 层架构顺序完成：
1. migration -> 建表
2. 提示我运行脚本生成 entity
3. repo -> 建立对应表的 dto 和单表 service
4. service -> 新增业务 api 与 dto
5. web-server -> handler + route + dto，通过调用 service 输出结果

禁止修改 db_core。
禁止在 repo 之外引入 sea-orm。
禁止手写 entity。
```

## 只新增 HTTP 接口的 AI 提示模板

```text
请阅读 ai_protocols/WEB_SERVER_GUIDE.md。

需要在 web-server 中新增 HTTP 接口：
- 路径：<METHOD> /<path>
- 功能：<描述>
- 请求参数：<Query/Body 参数>
- 响应格式：<响应结构>
- 调用的业务逻辑：<service crate 中的 API>

要求：
1. dto/<domain>.rs 定义 HTTP 请求/响应 DTO
2. handlers/<domain>.rs 实现 Handler
3. routes/<domain>.rs 注册路由
4. 请求 DTO 使用 validator 做参数校验
5. 业务错误通过 from_biz_error 转换
6. 成功响应使用 IntoCommonResponse
```

## 错误码约定

错误码集中放在：

```text
crates/error-code/src/
```

auth 域错误码在：

```text
crates/error-code/src/auth.rs
```

业务层使用：

```rust
BizError::new(auth::USER_NOT_FOUND, "...")
```

web-server 参数校验链路：

```text
web-server/src/dto/*.rs
  validator rule code 字符串

crates/error-code/src/auth.rs
  validation_rule_code(rule) -> Option<i16>
  user_message(code) -> Option<&'static str>

web-server/src/error/mod.rs
  ValidationErrors -> CommonError { code, message }
```

也就是说，想查“哪个校验错误对应哪个码”和“这个 code 对应什么用户文案”，优先看 `crates/error-code/src/auth.rs`。

注意：HTTP 响应体里的 `message` 是给开发调试和日志看的，不作为前端用户展示文案。前端用户文案应该由 `code` 映射得到；`error-code` 里的 `user_message(code)` 是这份映射的基准说明。

## AI 协议文档索引

- `ai_protocols/TABLE_ADDING_PROTOCOL.md`：完整表 + repo + service + web-server 开发流程
- `ai_protocols/MIGRATION_GUIDE.md`：migration 编写规范
- `ai_protocols/REPO_GUIDE.md`：单表 repo 开发规范
- `ai_protocols/SERVICE_GUIDE.md`：业务 service 编排规范
- `ai_protocols/WEB_SERVER_GUIDE.md`：HTTP DTO、handler、route 规范
- `ai_protocols/ERROR_CODE_GUIDE.md`：错误码设计规范

## 重要约束

- 不使用 Swagger / OpenAPI / utoipa
- 不在 `web-server` 和 `service` 中引入 `sea-orm`
- 不手写 `crates/repo/src/entity`
- 不修改 `db_core`
- 不手写魔法错误码，必须从 `error-code` crate 引用
- HTTP 响应体里的 `message` 只用于调试和日志，前端展示应以 `code` 对应的用户文案为准

## 验证

```bash
cargo fmt --check
cargo test
cargo test -p auth-server
```

## 部署构建

本仓库提供两种 Linux 构建产物。裸二进制 bundle：

```bash
make build-linux-bundle
```

生成产物会包含 `auth-server` 二进制和配置模板。

Linux x86_64 Docker 离线部署包：

```bash
# 提前准备以下两个 linux/amd64 镜像；构建脚本自身不会下载镜像
docker pull --platform linux/amd64 rust:1.94-alpine3.22
docker pull --platform linux/amd64 postgres:16-alpine
make build-docker-release
```

该命令会构建 `linux/amd64` 的 auth 运行镜像，并生成：

```text
dist/auth-server-linux-x86_64-docker.tar.gz
```

构建脚本使用本机 Cargo 缓存并通过 `--offline`、`--network=none` 和
`--pull=false` 禁止网络访问；缺少前置镜像或 Cargo 缓存时会直接报错。
产物不包含 Rust 编译环境或 builder 镜像，但包含 auth 服务必需的 PostgreSQL
运行镜像。目标机只需安装 Docker Engine 和 Docker Compose plugin。解压后
运行 `./install.sh`，脚本会加载离线镜像、启动数据库、执行迁移并启动服务。
详细说明见 `deploy/runtime/README.md`。
