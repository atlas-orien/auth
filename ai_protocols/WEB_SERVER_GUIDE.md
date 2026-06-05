# Web Server 层开发协议 (AI 专用)

> 本文档是 **执行规约**，AI 必须严格按步骤在 `crates/web-server` 中添加或修改 HTTP 接口。
> `web-server` 负责 HTTP 请求响应、参数校验和身份认证。它直接调用 `service` 层的业务 API。

---

## 一、核心原则

1. **绝对禁止数据库操作**：此层不允许出现 `sea-orm` 或任何数据库特有类型（比如 `DbErr`、`Model`）。所有业务逻辑都应代理给 `service` 层的 API。
2. **运行时接口优先**：此层只维护 Axum 路由、DTO、Handler 和响应转换，不引入 OpenAPI/Swagger 文档依赖。
3. **分层错误处理**：业务层的 `BizError` 必须转换为标准 HTTP 错误响应；参数校验错误必须返回 400 和统一错误码。
4. **强类型约束**：请求参数需通过 `validator` 和 `serde` 在进入业务逻辑前完成校验。
5. **统一响应结构**：成功响应使用 `toolcraft_axum_kit::IntoCommonResponse`；错误响应使用 `toolcraft_axum_kit::CommonError`。

---

## 二、标准开发流程

当需要新增一个业务模块（例如 `user_biz`）的 HTTP 接口时，请遵循以下步骤：

### Step 1: 编写 Web DTO (`dto/xxx.rs`)

在 `crates/web-server/src/dto/user_biz.rs` 中定义 HTTP 专用的请求和响应结构，并按需加上 `validator` 注解：

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 20, message = "username length must be between 3 and 20"))]
    pub username: String,

    #[validate(email(message = "email must be a valid email address"))]
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
}
```

### Step 2: 编写 Handler (`handlers/xxx.rs`)

在 `crates/web-server/src/handlers/user_biz.rs` 中编写处理函数。通过依赖注入（如 `get_default_ctx` 或提取器）获取所需上下文，调用 `service` 层，然后返回 `toolcraft_axum_kit::ResponseResult`。

```rust
use axum::{extract::Query, Json};
use toolcraft_axum_kit::{ApiError, IntoCommonResponse, ResponseResult};
use validator::Validate;

use service::api::user_biz::UserBizApi;

use crate::{
    dto::user_biz::*,
    error::{Error, from_biz_error},
    statics::db_manager::get_default_ctx,
};

pub async fn create_user_handler(
    Json(req): Json<CreateUserRequest>,
) -> ResponseResult<UserResponse> {
    req.validate()
        .map_err(Error::Validation)
        .map_err(ApiError::from)?;

    let api = UserBizApi::new(get_default_ctx());
    let biz_resp = api
        .create_user(service::dto::user_biz::CreateUserRequest {
            username: req.username,
            email: req.email,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(UserResponse {
        id: biz_resp.id,
        username: biz_resp.username,
        email: biz_resp.email,
    }
    .into_common_response()
    .to_json())
}
```

### Step 3: 注册 Routes (`routes/xxx.rs`)

在 `crates/web-server/src/routes/user_biz.rs` 中定义子路由结构：

```rust
use axum::{Router, routing::post};

use crate::handlers::user_biz::create_user_handler;

pub fn user_biz_routes() -> Router {
    Router::new().route("/users", post(create_user_handler))
}
```

### Step 4: 挂载到主路由 (`routes/mod.rs`)

在 `crates/web-server/src/routes/mod.rs` 中将子路由挂载上去：

```rust
mod user_biz;

use axum::{Extension, Router};
use toolcraft_axum_kit::middleware::cors::create_cors;
use toolcraft_jwt::Jwt;

pub fn create_routes(jwt: std::sync::Arc<Jwt>) -> Router {
    let cors = create_cors();

    Router::new()
        .nest("/user-biz", user_biz::user_biz_routes())
        .layer(Extension(jwt))
        .layer(cors)
}
```

### Step 5: 导出模块

在 `crates/web-server/src/dto/mod.rs` 和 `crates/web-server/src/handlers/mod.rs` 中声明对应的新模块：

```rust
pub mod user_biz;
```

---

## 三、自检清单

- [ ] 是否确保此层没有引入 `sea-orm` 或调用任何 `repo` 层的直接方法？
- [ ] DTO 是否使用 `serde` 做请求/响应序列化？
- [ ] 请求结构是否使用 `validator` 做入参校验？
- [ ] Handler 是否将 `ValidationErrors` 映射为 `Error::Validation`？
- [ ] 业务错误是否通过 `from_biz_error` 转换？
- [ ] 成功响应是否通过 `IntoCommonResponse` 包装？
- [ ] 是否没有新增 `utoipa`、`utoipa-swagger-ui`、`utoipa-axum` 或 Swagger/OpenAPI 相关代码？
