# Auth Service Linux x86_64 Offline Bundle

This bundle contains the `auth-server` and PostgreSQL runtime images for Linux
x86_64. It does not contain a Rust toolchain, source code, or the builder
image. The target machine does not need Rust or network access.

## Requirements

- Linux x86_64
- Docker Engine
- Docker Compose plugin (`docker compose`)

## Install

```bash
tar -xzf auth-server-linux-x86_64-docker.tar.gz
cd auth-server-linux-x86_64-docker
./install.sh
```

The installer loads the auth and PostgreSQL images, creates random PostgreSQL
and internal API secrets, applies database migrations, uses the
deployment-specific JWT keys included by the packager, and starts the service.
Existing configuration, keys, logs, and database data are preserved when it is
run again.

The default address is `http://SERVER_IP:29001`. Change the host port in
`.env` before rerunning `./install.sh` if needed.

## Configuration

Update the avatar object-storage endpoint, bucket, JWT issuer, and JWT audience
in `config/services.toml` for the deployment environment. If the PostgreSQL
password is changed later, update both `.env` and the database URL in
`config/services.toml`.

After changing configuration, restart the service:

```bash
docker compose up -d auth
```

Useful commands:

```bash
docker compose ps
docker compose logs -f auth
docker compose down
```

`docker compose down` preserves the PostgreSQL named volume. Deleting that
volume deletes the auth database and is intentionally not part of the installer.
