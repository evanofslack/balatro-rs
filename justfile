default: dev

test:
    cargo test

lint:
    cargo clippy --all-targets

fmt:
    cargo fmt

check:
    cargo check

build:
    cargo build

python-dev:
    cd pylatro && maturin develop

bench:
    cargo bench -p balatro-rs

dev: fmt check

cli:
    cargo run -p balatro-cli

tui:
    cargo run -p balatro-tui

tui-load FILE:
    cargo run -p balatro-tui -- --load {{FILE}}
