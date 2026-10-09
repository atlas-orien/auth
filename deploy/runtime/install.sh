#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

IMAGE_TAR="${IMAGE_TAR:-auth-server-images.tar}"
CONFIG_FILE="${CONFIG_FILE:-deploy.env}"

fail() {
  echo "error: $*" >&2
  exit 1
}

escape_toml_value() {
  printf '%s' "$1" | sed 's/[\\&|"]/\\&/g'
}

if [ "${ALLOW_NON_LINUX:-0}" != "1" ]; then
  case "$(uname -m)" in
    x86_64|amd64) ;;
    *) fail "this bundle requires Linux x86_64; detected $(uname -m)" ;;
  esac
  [ "$(uname -s)" = "Linux" ] || fail "this installer only supports Linux"
fi
command -v docker >/dev/null 2>&1 || fail "Docker is not installed"
docker info >/dev/null 2>&1 || fail "Docker daemon is unavailable or access is denied"
docker compose version >/dev/null 2>&1 || fail "Docker Compose plugin is not installed"
[ -f "$IMAGE_TAR" ] || fail "missing $IMAGE_TAR"
[ -f "$CONFIG_FILE" ] || fail "missing deployment configuration: $CONFIG_FILE"

set -a
. "./$CONFIG_FILE"
set +a

: "${POSTGRES_USER:?POSTGRES_USER is required in deploy.env}"
: "${POSTGRES_PASSWORD:?POSTGRES_PASSWORD is required in deploy.env}"
: "${POSTGRES_DB:?POSTGRES_DB is required in deploy.env}"
: "${POSTGRES_DATA_DIR:?POSTGRES_DATA_DIR is required in deploy.env}"
: "${INTERNAL_AUTH_TOKEN:?INTERNAL_AUTH_TOKEN is required in deploy.env}"
: "${JWT_ISSUER:?JWT_ISSUER is required in deploy.env}"
: "${JWT_AUDIENCE:?JWT_AUDIENCE is required in deploy.env}"

if [ "$POSTGRES_PASSWORD" = "replace_with_random_password" ] || \
  [ "$INTERNAL_AUTH_TOKEN" = "replace_with_random_token" ]; then
  fail "deploy.env still contains secret placeholders; regenerate the deployment package"
fi

case "$POSTGRES_USER$POSTGRES_PASSWORD$POSTGRES_DB" in
  *[!A-Za-z0-9._-]*) fail "PostgreSQL user, password, and database may only contain letters, digits, dot, underscore, and hyphen" ;;
esac

mkdir -p "$POSTGRES_DATA_DIR" config/key logs

postgres_user="$(escape_toml_value "$POSTGRES_USER")"
postgres_password="$(escape_toml_value "$POSTGRES_PASSWORD")"
postgres_db="$(escape_toml_value "$POSTGRES_DB")"
internal_auth_token="$(escape_toml_value "$INTERNAL_AUTH_TOKEN")"
jwt_issuer="$(escape_toml_value "$JWT_ISSUER")"
jwt_audience="$(escape_toml_value "$JWT_AUDIENCE")"

{
  printf '%s\n' '[http]' 'port = 19878' '' '[internal]'
  printf 'auth_header = "x-internal-token"\nauth_token = "%s"\n\n' "$internal_auth_token"
  printf '[jwt]\nissuer = "%s"\naudience = "%s"\n' "$jwt_issuer" "$jwt_audience"
  printf '%s\n' 'key_dir = "config/key"' 'access_token_duration = 10800' 'refresh_token_duration = 604800' 'access_key_validate_exp = true' 'refresh_key_validate_exp = true' '' '[[db]]' 'name = "default"'
  printf 'url = "postgres://%s:%s@postgres:5432/%s"\n' "$postgres_user" "$postgres_password" "$postgres_db"
  printf '%s\n' 'max_connections = 10' 'min_connections = 1' 'connect_timeout = 30' 'idle_timeout = 600' 'sql_logging = false'
} > config/services.toml
chmod 600 config/services.toml

echo "==> Loading Linux x86_64 Docker images"
docker load -i "$IMAGE_TAR"

echo "==> Starting PostgreSQL and applying migrations"
docker compose --env-file "$CONFIG_FILE" up -d postgres
docker compose --env-file "$CONFIG_FILE" --profile tools run --rm migrate

echo "==> Starting auth service"
docker compose --env-file "$CONFIG_FILE" up -d auth

echo "Auth service: http://localhost:${AUTH_PORT:-29001}"
echo "Runtime config: $SCRIPT_DIR/config/services.toml"
