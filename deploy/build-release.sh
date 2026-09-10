#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

PLATFORM="linux/amd64"
IMAGE="auth-server:local"
BUILDER_IMAGE="rust:1.94-alpine3.22"
POSTGRES_IMAGE="postgres:16-alpine"
VENDOR_DIR="$PROJECT_DIR/deploy/vendor"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: required command not found: $1" >&2
    exit 1
  fi
}

require_command cargo
require_command docker

if [ "$PLATFORM" != "linux/amd64" ]; then
  echo "error: this release bundle only supports PLATFORM=linux/amd64" >&2
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "error: Docker daemon is not available" >&2
  exit 1
fi

require_local_amd64_image() {
  local image="$1"
  local command="$2"

  if ! docker image inspect "$image" >/dev/null 2>&1; then
    echo "error: required local image is missing: $image" >&2
    echo "prepare both images yourself before running this script:" >&2
    echo "  docker pull --platform linux/amd64 $BUILDER_IMAGE" >&2
    echo "  docker pull --platform linux/amd64 $POSTGRES_IMAGE" >&2
    exit 1
  fi

  if ! docker run --rm --pull=never --platform "$PLATFORM" \
    --entrypoint /bin/sh "$image" -c "$command" >/dev/null; then
    echo "error: local linux/amd64 variant is unavailable for $image" >&2
    echo "prepare both images yourself:" >&2
    echo "  docker pull --platform linux/amd64 $BUILDER_IMAGE" >&2
    echo "  docker pull --platform linux/amd64 $POSTGRES_IMAGE" >&2
    exit 1
  fi
}

# docker image inspect selects the host-default arm64 child of a multi-platform
# index on Apple Silicon, so validate the locally stored amd64 child directly.
require_local_amd64_image "$BUILDER_IMAGE" "rustc --version"
require_local_amd64_image "$POSTGRES_IMAGE" "postgres --version"

cleanup() {
  rm -rf "$VENDOR_DIR"
}
trap cleanup EXIT

cd "$PROJECT_DIR"
rm -rf "$VENDOR_DIR"

echo "==> Vendoring Cargo dependencies from the local cache"
cargo vendor --offline --locked --versioned-dirs "$VENDOR_DIR" >/dev/null

echo "==> Building ${IMAGE} for ${PLATFORM} without network access"
docker build \
  --platform "$PLATFORM" \
  --pull=false \
  --network=none \
  --tag "$IMAGE" \
  --file deploy/Dockerfile \
  .

architecture="$(docker image inspect --format '{{.Architecture}}' "$IMAGE")"
if [ "$architecture" != "amd64" ]; then
  echo "error: $IMAGE has architecture $architecture, expected amd64" >&2
  exit 1
fi
echo "verified: $IMAGE ($architecture)"

"$SCRIPT_DIR/package-offline.sh"
