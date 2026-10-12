#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

IMAGE="auth-server:local"
POSTGRES_IMAGE="postgres:16-alpine"
OUTPUT_ROOT="${OUTPUT_ROOT:-$PROJECT_DIR/dist}"
BUNDLE_NAME="${BUNDLE_NAME:-auth-server-linux-x86_64-docker}"
BUNDLE_DIR="$OUTPUT_ROOT/$BUNDLE_NAME"
ARCHIVE_PATH="$OUTPUT_ROOT/$BUNDLE_NAME.tar.gz"
IMAGE_TAR="auth-server-images.tar"

random_hex() {
  od -An -N 24 -tx1 /dev/urandom | tr -d ' \n'
}

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "error: missing Docker image: $IMAGE" >&2
  echo "run deploy/build-release.sh first" >&2
  exit 1
fi

architecture="$(docker image inspect --format '{{.Architecture}}' "$IMAGE")"
if [ "$architecture" != "amd64" ]; then
  echo "error: $IMAGE has architecture $architecture, expected amd64" >&2
  exit 1
fi

if ! docker run --rm --pull=never --platform linux/amd64 \
  --entrypoint /bin/sh "$POSTGRES_IMAGE" -c 'postgres --version' >/dev/null; then
  echo "error: local linux/amd64 variant is unavailable for $POSTGRES_IMAGE" >&2
  exit 1
fi

mkdir -p "$OUTPUT_ROOT"
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"

echo "==> Saving Docker images"
docker save --platform linux/amd64 \
  "$IMAGE" \
  "$POSTGRES_IMAGE" \
  -o "$BUNDLE_DIR/$IMAGE_TAR"

cp -R "$SCRIPT_DIR/runtime/." "$BUNDLE_DIR/"
chmod +x "$BUNDLE_DIR/install.sh"
chmod +x "$BUNDLE_DIR/stop.sh"
chmod +x "$BUNDLE_DIR/start.sh"
chmod +x "$BUNDLE_DIR/restart.sh"
chmod +x "$BUNDLE_DIR/reset-database.sh"

echo "==> Generating deployment secrets"
postgres_password="$(random_hex)"
internal_auth_token="$(random_hex)"
sed \
  -e "s/replace_with_random_password/$postgres_password/" \
  -e "s/replace_with_random_token/$internal_auth_token/" \
  "$BUNDLE_DIR/deploy.env" > "$BUNDLE_DIR/deploy.env.tmp"
mv "$BUNDLE_DIR/deploy.env.tmp" "$BUNDLE_DIR/deploy.env"
chmod 600 "$BUNDLE_DIR/deploy.env"

echo "==> Generating deployment-specific JWT keys"
"$PROJECT_DIR/scripts/gen_jwt_keys.sh" "$BUNDLE_DIR/config/key"

echo "==> Creating $ARCHIVE_PATH"
TAR_ARGS=()
if tar --help 2>/dev/null | grep -q -- '--no-xattrs'; then
  TAR_ARGS+=(--no-xattrs)
fi
if tar --help 2>/dev/null | grep -q -- '--no-mac-metadata'; then
  TAR_ARGS+=(--no-mac-metadata)
fi

if [ "${#TAR_ARGS[@]}" -gt 0 ]; then
  COPYFILE_DISABLE=1 tar "${TAR_ARGS[@]}" -czf "$ARCHIVE_PATH" -C "$OUTPUT_ROOT" "$BUNDLE_NAME"
else
  COPYFILE_DISABLE=1 tar -czf "$ARCHIVE_PATH" -C "$OUTPUT_ROOT" "$BUNDLE_NAME"
fi

echo "==> Done"
echo "Offline bundle: $ARCHIVE_PATH"
