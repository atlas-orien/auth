# Auth 服务

Auth 是项目的用户身份服务，负责注册、登录、JWT 签发、用户资料和邮箱/密码管理。用户资料中的头像保存为对象存储 key；Auth 不保存 S3 endpoint 或 bucket 地址。

## 功能

- 用户注册、登录与 access/refresh token 刷新
- 当前用户资料、邮箱、密码和邮箱验证
- 用户头像 object key 保存及版本标记
- 向内部服务提供 JWT 验证公钥、issuer 和 audience

## 接口

Auth 默认监听 `19878`，实际端口由 `[http].port` 决定。受保护接口使用：

```http
Authorization: Bearer <access_token>
```

| 方法 | 路径 | 用途 |
| --- | --- | --- |
| `POST` | `/auth/session/register` | 注册 |
| `POST` | `/auth/session/login` | 登录并取得 access/refresh token |
| `POST` | `/auth/session/refresh_token` | 刷新 access token |
| `GET` | `/auth/user/me` | 当前用户资料 |
| `PUT` | `/auth/user/profile` | 更新资料和头像 key |
| `PUT` | `/auth/user/email` | 更新邮箱 |
| `PUT` | `/auth/user/password` | 更新密码 |
| `POST` | `/auth/user/email/verify` | 验证邮箱 |

内部接口使用 `[internal].auth_header` 和 `[internal].auth_token` 认证：

| 方法 | 路径 | 用途 |
| --- | --- | --- |
| `GET` | `/internal/jwt_verify_config` | 返回 access 公钥、issuer 和 audience |
| `GET` | `/internal/users/by_identifier` | 按账号查询用户 |
| `GET` | `/internal/users/{display_user_id}/uuid` | 将展示 ID 转成 UUID |

## JWT 配置

`config/services.toml` 中的 `[jwt]` 配置签发参数。默认密钥目录是 `config/key`：

```toml
[jwt]
issuer = "auth-server"
audience = "test"
key_dir = "config/key"
access_token_duration = 10800
refresh_token_duration = 604800
access_key_validate_exp = true
refresh_key_validate_exp = true
```

在项目根目录生成 Ed25519 access/refresh 密钥：

```bash
bash scripts/gen_jwt_keys.sh
```

Auth 使用私钥签发 token；file-service 通过 `/internal/jwt_verify_config` 获取 access 公钥及 issuer/audience。`[internal].auth_token` 是服务间认证 token，不是 JWT 签名密钥。不要重新生成已部署环境的密钥，否则旧 token 会失效。

## 头像与 File Service

头像注册和资料更新只接收 object key，不接收 S3 URL。Auth 在数据库中保存 key 和版本；用户资料响应将它们分开返回：

```json
{
  "avatar": {
    "object_key": "avatars/{user_scope}",
    "version": 1791540795
  }
}
```

客户端将 `object_key` 传给 file-service，带 Auth access token 请求：

```http
GET /file/sign/access?bucket_id=avatar&key=avatars%2F{user_scope}
Authorization: Bearer <access_token>
```

客户端使用 file-service 响应中的 `download_url` 显示头像。Auth 不需要知道 COS 域名、bucket 名或 S3 凭据。

## 本地运行

1. 复制 `config/services-example.toml` 为 `config/services.toml`，配置 PostgreSQL、JWT 和内部 token。
2. 确保数据库已创建，并让 `[db].url` 与迁移使用的 `DATABASE_URL` 指向同一数据库。
3. 设置 `DATABASE_URL` 后运行 `make migrate-up` 创建表结构。
4. 运行 `cargo run -p auth-server`。

`scripts/init_db.sh` 只创建数据库，不会创建表。`make migrate-fresh` 和 `make migrate-refresh` 会重置数据库，请勿用于保留现有数据的环境。详细迁移命令见 `Makefile`。

## 离线部署

在本机准备好 `linux/amd64` 的 Rust builder 和 PostgreSQL 镜像后运行：

```bash
./deploy/build-release.sh
```

该脚本从当前工作区构建 Auth 镜像并生成 `dist/auth-server-linux-x86_64-docker.tar.gz`，构建阶段使用本地 Cargo 缓存和 Docker 离线设置。目标服务器的安装步骤见 [deploy/runtime/README.md](deploy/runtime/README.md)。
