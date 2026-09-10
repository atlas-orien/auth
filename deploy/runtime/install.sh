#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

IMAGE_TAR="${IMAGE_TAR:-auth-server-images.tar}"

fail() {
  echo "error: $*" >&2
  exit 1
}

random_hex() {
  od -An -N 24 -tx1 /dev/urandom | tr -d ' \n'
}

case "$(uname -m)" in
  x86_64|amd64) ;;
  *) fail "this bundle requires Linux x86_64; detected $(uname -m)" ;;
esac

[ "$(uname -s)" = "Linux" ] || fail "this installer only supports Linux"
command -v docker >/dev/null 2>&1 || fail "Docker is not installed"
docker info >/dev/null 2>&1 || fail "Docker daemon is unavailable or access is denied"
docker compose version >/dev/null 2>&1 || fail "Docker Compose plugin is not installed"
[ -f "$IMAGE_TAR" ] || fail "missing $IMAGE_TAR"

if [ ! -f .env ]; then
  postgres_password="$(random_hex)"
  {
    echo "AUTH_PORT=29001"
    echo "POSTGRES_PASSWORD=$postgres_password"
  } > .env
  chmod 600 .env
fi

set -a
. ./.env
set +a

: "${POSTGRES_PASSWORD:?POSTGRES_PASSWORD is required in .env}"

if [ "$POSTGRES_PASSWORD" = "replace_with_random_password" ]; then
  fail "replace the example POSTGRES_PASSWORD in .env"
fi

case "$POSTGRES_PASSWORD" in
  *[!A-Za-z0-9._-]*) fail "POSTGRES_PASSWORD may only contain letters, digits, dot, underscore, and hyphen" ;;
esac

mkdir -p config/key logs

if [ ! -f config/services.toml ]; then
  internal_auth_token="$(random_hex)"
  sed \
    -e "s/__POSTGRES_PASSWORD__/$POSTGRES_PASSWORD/g" \
    -e "s/__INTERNAL_AUTH_TOKEN__/$internal_auth_token/g" \
    config/services.toml.template > config/services.toml
  chmod 600 config/services.toml
fi

echo "==> Loading Linux x86_64 Docker images"
docker load -i "$IMAGE_TAR"

echo "==> Starting PostgreSQL and applying migrations"
docker compose up -d postgres
docker compose --profile tools run --rm migrate

echo "==> Starting auth service"
docker compose up -d auth

echo "Auth service: http://localhost:${AUTH_PORT:-29001}"
echo "Runtime config: $SCRIPT_DIR/config/services.toml"
