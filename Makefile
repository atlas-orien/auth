.DEFAULT_GOAL := all

.PHONY: all help fresh-db postgres init build test examples clean cross-setup-linux cross-build-web-server-linux build-linux-bundle build-docker-release package-docker-offline show-docker-bundle upload-docker-bundle

CROSS_DOCKER_PLATFORM ?= linux/amd64
CROSS_LINUX_TARGET ?= x86_64-unknown-linux-gnu

all: build-docker-release upload-docker-bundle

# Default target
help:
	@echo "Available commands:"
	@echo "  make fresh-db    - Refresh database and generate entities"
	@echo "  make postgres    - Manage PostgreSQL with Docker"
	@echo "  make init        - Initialize project"
	@echo ""
	@echo "  make build       - Build all crates"
	@echo "  make test        - Run all tests"
	@echo "  make examples    - Build all examples"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make cross-setup-linux            - Install non-host Rust toolchain for cross"
	@echo "  make cross-build-web-server-linux - Cross-build web-server for Linux"
	@echo "  make build-linux-bundle           - Build Linux binary and package config"
	@echo "  make build-docker-release         - Build amd64 images and offline deploy tarball"
	@echo "  make package-docker-offline       - Package existing amd64 Docker images"
	@echo "  make show-docker-bundle           - Show the offline deploy tarball path"
	@echo "  make upload-docker-bundle         - Upload the offline tarball to the default server"
	@echo ""
	@echo "  make migrate-up     - Run migrations"
	@echo "  make migrate-down   - Rollback migrations"
	@echo "  make migrate-fresh  - Fresh migrations (drop all & rerun)"

# Scripts
fresh-db:
	@./scripts/fresh_db.sh

postgres:
	@./scripts/postgres.sh

init:
	@./scripts/init.sh

# Cargo commands
build:
	cargo build

test:
	cargo test

examples:
	cargo build --examples

clean:
	cargo clean

# Cross compile helpers for Apple Silicon -> Linux/amd64
cross-setup-linux:
	rustup toolchain add nightly-$(CROSS_LINUX_TARGET) --profile minimal --force-non-host

cross-build-web-server-linux:
	CROSS_CONTAINER_OPTS="--platform $(CROSS_DOCKER_PLATFORM)" cross build --target $(CROSS_LINUX_TARGET) --release -p auth-server

build-linux-bundle:
	@./scripts/build_linux_bundle.sh

build-docker-release:
	@./deploy/build-release.sh
	@echo "Deploy tar: $(abspath dist/auth-server-linux-x86_64-docker.tar.gz)"

package-docker-offline:
	@./deploy/package-offline.sh
	@echo "Deploy tar: $(abspath dist/auth-server-linux-x86_64-docker.tar.gz)"

show-docker-bundle:
	@if [ -f "dist/auth-server-linux-x86_64-docker.tar.gz" ]; then \
		echo "Deploy tar: $(abspath dist/auth-server-linux-x86_64-docker.tar.gz)"; \
	else \
		echo "Deploy tar has not been built yet"; \
		exit 1; \
	fi

upload-docker-bundle:
	@./deploy/upload.sh

# Migration commands
migrate-up:
	sea-orm-cli migrate up -d crates/migration

migrate-down:
	sea-orm-cli migrate down -d crates/migration

migrate-fresh:
	sea-orm-cli migrate fresh -d crates/migration

migrate-refresh:
	sea-orm-cli migrate refresh -d crates/migration

# Generate new migration
# Usage: make migrate-gen NAME=create_users
migrate-gen:
	@if [ -z "$(NAME)" ]; then \
		echo "Error: NAME is required. Usage: make migrate-gen NAME=create_users"; \
		exit 1; \
	fi
	sea-orm-cli migrate generate $(NAME) -d crates/migration

# Generate entities
generate-entity:
	sea-orm-cli generate entity -o crates/pg-tables/src/entity --with-serde both

# Run examples
example-basic:
	cargo run --example basic_usage

example-multi:
	cargo run --example multi_database
