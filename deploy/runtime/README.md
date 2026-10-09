# Auth 服务 Linux x86_64 离线部署包

该部署包包含 `auth-server` 和 PostgreSQL 的 Linux x86_64 运行镜像，不包含
Rust 编译环境、源代码或构建镜像。目标服务器不需要安装 Rust，也不需要访问网络。

## 环境要求

- Linux x86_64
- Docker Engine
- Docker Compose plugin（`docker compose`）

## 安装

```bash
tar -xzf auth-server-linux-x86_64-docker.tar.gz
cd auth-server-linux-x86_64-docker
# 安装前只需按需修改这一份配置文件。
vi deploy.env
./install.sh
```

安装脚本会加载 auth 和 PostgreSQL 镜像、执行数据库迁移，并启动服务。重复执行
安装脚本会保留已有配置、JWT 密钥、日志和数据库数据。PostgreSQL 密码和内部 API
token 在打包时随机生成，已写入 `deploy.env`。

默认访问地址为 `http://服务器IP:39999`。需要调整端口时，修改 `deploy.env` 后
重新执行 `./install.sh`。

## 配置

所有运行时配置均在 `deploy.env` 中：服务端口、PostgreSQL 用户/密码/数据库名、
数据目录、内部 API token，以及 JWT 的 issuer/audience。Auth 只保存用户头像的
object key 和版本；客户端使用 access token 请求 file-service 获取实际图片地址。
不要直接修改安装脚本生成的 `config/services.toml`。

修改配置后重新运行安装脚本：

```bash
./restart.sh
```

常用命令：

```bash
docker compose --env-file deploy.env ps
docker compose --env-file deploy.env logs -f auth
./start.sh
./restart.sh
./stop.sh
```

`./stop.sh` 会保留 PostgreSQL 数据目录；删除 `deploy.env` 中
`POSTGRES_DATA_DIR` 指向的目录才会删除 auth 数据库。
