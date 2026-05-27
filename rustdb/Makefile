.PHONY: build test dev release clean docker help

help:
	@echo "Available targets:"
	@echo "  build          - Build the project"
	@echo "  test           - Run all tests"
	@echo "  test-unit      - Run unit tests"
	@echo "  test-integration - Run integration tests"
	@echo "  dev            - Run in development mode"
	@echo "  release        - Build release version"
	@echo "  clean          - Clean build artifacts"
	@echo "  docker-build   - Build Docker image"
	@echo "  docker-run     - Run Docker container"
	@echo "  benchmark      - Run benchmarks"

build:
	cargo build

test:
	cargo test --all

test-unit:
	cargo test --lib

test-integration:
	cargo test --test '*' -- --test-threads=1

dev:
	cargo run --manifest-path rustdb-server/Cargo.toml

release:
	cargo build --release

clean:
	cargo clean

docker-build:
	docker build -t rustdb:latest .

docker-run:
	docker-compose up

benchmark:
	cargo bench --all

fmt:
	cargo fmt --all

lint:
	cargo clippy --all -- -D warnings

check:
	cargo check --all
