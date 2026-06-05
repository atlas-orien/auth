# 数据库与接口全栈开发协议 (AI 专用)

> 本文档是 **执行规约**，AI 必须严格按步骤执行。
> 不允许引入未明确要求的结构、逻辑或抽象。

---

## 一、项目架构与职责边界

```
┌──────────────────────────────────────────────────────┐
│                    web-server                        │  Layer 5: HTTP API
├──────────────────────────────────────────────────────┤
│                     service                          │  Layer 4: Business API
├──────────────────────────────────────────────────────┤
│                      repo                            │  Layer 3: Domain Services (隔离区)
├──────────────────────────────────────────────────────┤
│                    db_core                           │  Layer 2: Infrastructure
├──────────────────────────────────────────────────────┤
│                    migration                         │  Layer 1: Schema
└──────────────────────────────────────────────────────┘
```

### Layer 1: migration
| 项目 | 说明 |
|------|------|
| 职责 | 数据库表结构定义 |
| 允许 | 新增/修改 migration 文件 |
| 禁止 | 写业务逻辑、写 Rust struct（除 migration 所需） |

### Layer 2: db_core (代码中引用为 `db_core`)
| 项目 | 说明 |
|------|------|
| 职责 | 基础能力（连接池、错误、Repository trait、分页） |
| 允许 | 调用 `db_core` 提供的 API（如 DbContext、PgError、impl_repository! 等） |
| 禁止 | 修改 `db_core` 的代码、接口或行为 |
| 规则 | **永远不动**，新增表/业务不需要修改此层 |

### Layer 3: repo (原 pg-tables)
| 项目 | 说明 |
|------|------|
| 职责 | 表级事实模型 + 单表 Service，隔离 `sea-orm` |
| 包含 | entity（自动生成）、dto、service |
| 规则 | 单表操作，不跨表 JOIN，不校验外键。**这是整个业务逻辑中唯一允许引入 `sea-orm` 的 crate**。 |

### Layer 4: service (原 demo-db)
| 项目 | 说明 |
|------|------|
| 职责 | 业务 API 层，编排多个 Repo Service |
| 包含 | api、dto（业务级 DTO） |
| 规则 | 可跨表组合，处理业务语义，不直接操作数据库。**绝对禁止引入 `sea-orm`**。 |

### Layer 5: web-server
| 项目 | 说明 |
|------|------|
| 职责 | HTTP 接口适配 |
| 包含 | routes、handlers、dto（请求/响应）、error |
| 规则 | 参数校验、错误转换、调用 service API。**绝对禁止引入 `sea-orm` 或任何数据库类型**。 |

---

## 二、AI 能做与不能做

### 允许 AI 执行的任务

| 任务类型 | 涉及 Crate | 说明 |
|----------|-----------|------|
| 新增表 | migration, repo | 按协议流程执行，先 migration 再生成 entity |
| 新增单表操作 | repo | 仅操作单表，吐出 DTO |
| 新增/修改业务 API | service | 编排多个 Repo，处理业务逻辑 |
| 新增/修改 HTTP 接口 | web-server | 路由、Handler、DTO、参数校验 |

### 禁止 AI 执行的任务

| 禁止行为 | 原因 |
|----------|------|
| 修改 db_core | 基础设施层稳定，不随业务变化 |
| 在 repo 中跨表 JOIN | 违反单表原则，跨表逻辑应在 service 进行 |
| 在 repo 中校验外键存在 | 这是 service 的职责 |
| 在 repo 外使用 `sea-orm` | 破坏架构隔离层原则 |
| 手写 entity | entity 由 sea-orm-cli 生成，运行 `./scripts/fresh_db.sh` 即可 |

---

## 三、标准开发全流程

### Step 1: Migration（Layer 1）
使用 `sea-orm-cli migrate generate <table_name> -d crates/migration` 生成文件，并使用 `.if_not_exists()` 和 DSL 编写建表语句。详见 `MIGRATION_GUIDE.md`。

### Step 2: 生成 Entity
运行 `./scripts/fresh_db.sh` 自动重建数据库并在 `crates/repo/src/entity` 中生成 Rust 结构体。**千万不要手写 entity。**

### Step 3: Repo 单表开发（Layer 3）
在 `crates/repo/src/table/<table_name>` 目录下创建 `dto.rs` 和 `service.rs`。
使用 `db_core::impl_repository!` 宏包裹 entity，暴露只针对单表的 CRUD 方法，并负责将 `sea_orm::Model` 转换为干净的 DTO。详见 `REPO_GUIDE.md`。

### Step 4: Service 业务编排（Layer 4）
在 `crates/service/src/api/<domain_name>.rs` 创建业务接口，通过注入相关的 `repo` 的 Service 实例，调用单表接口完成复杂业务聚合和外键验证。不直接触碰 SQL。详见 `SERVICE_GUIDE.md`。

### Step 5: Web HTTP 暴露（Layer 5）
在 `crates/web-server/src/handlers/<domain_name>.rs` 编写 handler，组装请求参数并调用 `service` 层的方法。详见 `WEB_SERVER_GUIDE.md`。

---

## 四、自检清单

- [ ] `migration` 是否能无损回滚？
- [ ] `entity` 确保是通过脚本生成的，没有手改？
- [ ] `repo` 是不是业务层唯一引入了 `sea-orm` 的地方？
- [ ] `service` 是否没有任何直接的数据库驱动调用，纯靠拿 `repo` 的 DTO 拼装？
- [ ] `web-server` 的接口是否通过 `validator` 做好了入参拦截？
- [ ] `web-server` 里的 Error 是否成功将业务 `Result` 转成了合适的 HTTP 状态码？
