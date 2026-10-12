#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

CONFIG_FILE="${CONFIG_FILE:-deploy.env}"
case "$CONFIG_FILE" in
  /*) ;;
  *) CONFIG_FILE="$SCRIPT_DIR/$CONFIG_FILE" ;;
esac

fail() {
  echo "error: $*" >&2
  exit 1
}

command -v docker >/dev/null 2>&1 || fail "Docker is not installed"
docker compose version >/dev/null 2>&1 || fail "Docker Compose plugin is not installed"
[ -f "$CONFIG_FILE" ] || fail "deployment configuration not found: $CONFIG_FILE"

set -a
. "$CONFIG_FILE"
set +a

: "${POSTGRES_USER:?POSTGRES_USER is required in deploy.env}"
: "${POSTGRES_DB:?POSTGRES_DB is required in deploy.env}"

case "$POSTGRES_DB" in
  postgres|template0|template1) fail "refusing to reset PostgreSQL system database '$POSTGRES_DB'" ;;
esac

if ! docker compose --env-file "$CONFIG_FILE" ps --status running --services | grep -qx postgres; then
  fail "PostgreSQL container is not running; start and verify the intended database first"
fi

if [ ! -t 0 ]; then
  fail "run this script from an interactive terminal to confirm database deletion"
fi

echo "This will permanently delete all data in PostgreSQL database '$POSTGRES_DB'."
echo "The PostgreSQL data directory and other databases will be left untouched."
printf "Type the database name to continue: "
IFS= read -r confirmation
[ "$confirmation" = "$POSTGRES_DB" ] || fail "database name did not match; nothing was changed"

echo "==> Stopping Auth"
docker compose --env-file "$CONFIG_FILE" stop auth

echo "==> Dropping and recreating database '$POSTGRES_DB'"
docker compose --env-file "$CONFIG_FILE" exec -T postgres \
  dropdb --if-exists --force -U "$POSTGRES_USER" "$POSTGRES_DB"
docker compose --env-file "$CONFIG_FILE" exec -T postgres \
  createdb -U "$POSTGRES_USER" "$POSTGRES_DB"

echo "==> Applying migrations"
docker compose --env-file "$CONFIG_FILE" --profile tools run --rm migrate

echo "==> Starting Auth"
docker compose --env-file "$CONFIG_FILE" up -d auth
echo "Auth database '$POSTGRES_DB' has been reset and migrated."
