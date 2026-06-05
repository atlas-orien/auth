# auth

[中文](./README_CN.md) | **English**

A lightweight authentication service built with Rust. Provides user registration, login, JWT token management, and basic user profile operations via a REST API.

## Features

- User registration and login (by username or email)
- JWT access token + refresh token pair
- Token refresh endpoint
- Get current user info (`/me`)
- Update password
- Update email
- Email verification

## Quick Start

### 1. Setup Environment

```bash
# Start PostgreSQL (requires Docker)
make postgres

# Copy and edit config
cp config/services-example.toml config/services.toml

# Generate Ed25519 key pairs (access + refresh)
bash scripts/gen_jwt_keys.sh
```

Edit `config/services.toml` and update the database URL and JWT issuer/audience as needed.

### 2. Run Migrations

```bash
make migrate-up
```

### 3. Start the Server

```bash
cargo run -p auth-server
```

## API Endpoints

### Session (`/auth/session`)

| Method | Path              | Description               | Auth Required |
| ------ | ----------------- | ------------------------- | ------------- |
| POST   | `/register`       | Register a new user       | No            |
| POST   | `/login`          | Login, returns token pair | No            |
| POST   | `/refresh_token`  | Refresh access token      | No            |

### User (`/auth/user`)

| Method | Path             | Description          | Auth Required |
| ------ | ---------------- | -------------------- | ------------- |
| GET    | `/me`            | Get current user     | Yes (Bearer)  |
| PUT    | `/password`      | Update password      | Yes (Bearer)  |
| PUT    | `/email`         | Update email         | Yes (Bearer)  |
| PUT    | `/profile`       | Update display name and avatar | Yes (Bearer)  |
| POST   | `/email/verify`  | Verify email         | Yes (Bearer)  |

---

## Architecture

```text
┌──────────────────────────────────────────────────────┐
│                    web-server                        │  HTTP API (Axum)
├──────────────────────────────────────────────────────┤
│                     service                          │  Business Logic
├──────────────────────────────────────────────────────┤
│                      repo                            │  Data Access (SeaORM)
├──────────────────────────────────────────────────────┤
│               (External) db-core-rs                  │  Shared Core & Base Traits
├──────────────────────────────────────────────────────┤
│                    migration                         │  Schema (SeaORM Migrations)
└──────────────────────────────────────────────────────┘
```

## Database Schema

**users** table:

| Column         | Type        | Notes                      |
| -------------- | ----------- | -------------------------- |
| id             | string (PK) | Unique user ID             |
| display_user_id| string      | Public user ID, unique, nullable |
| username       | string      | Unique                     |
| display_name   | string      | Nullable                   |
| avatar         | string      | Nullable (URL)             |
| password       | string      | Hashed                     |
| email          | string      | Nullable, unique           |
| email_verified | boolean     | Default false              |
| disabled       | boolean     | Default false              |
| created_at     | timestamptz |                            |
| updated_at     | timestamptz |                            |

## Configuration (`config/services.toml`)

```toml
[http]
port = 19878

[jwt]
issuer = "auth-server"
audience = "test"
key_dir = "config/key"
access_token_duration = 10800   # 3 hours
refresh_token_duration = 604800 # 1 week

[[db]]
name = "default"
url = "postgres://postgres:123456@localhost:15432/auth"
```

## Tech Stack

| Component     | Technology          |
| ------------- | ------------------- |
| Runtime       | Tokio               |
| ORM           | SeaORM              |
| Web Framework | Axum 0.8            |
| JWT           | toolcraft-jwt       |
| Core Lib      | db-core-rs          |

## Development Commands

```bash
make help           # Show all commands
make postgres       # Start PostgreSQL container
make migrate-up     # Run pending migrations
make migrate-fresh  # Reset DB and run all migrations
make build          # Build all crates
```

## License

MIT or Apache-2.0
