default: dev

test:
    cargo test

test-core:
    cargo test -p balatro-rs

lint:
    cargo clippy --all-targets

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

check:
    cargo check

build:
    cargo build

build-release:
    cargo build --release

build-cli:
    cargo build -p balatro-cli

build-no-python:
    cargo build -p balatro-rs --no-default-features --features serde

python-dev:
    cd pylatro && maturin develop

bench:
    cargo bench -p balatro-rs

ci: fmt-check lint test

dev: fmt check
