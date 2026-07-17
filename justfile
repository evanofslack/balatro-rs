default: pre

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

pre: test fmt check lint 

python-dev:
    cd pylatro && maturin develop

bench:
    cargo bench -p balatro-rs

cli:
    cargo run -p balatro-cli

edit *ARGS:
    cargo run -p balatro-cli --bin balatro-edit -- {{ARGS}}

tui:
    cargo run -p balatro-tui

tui-load FILE:
    cargo run -p balatro-tui -- --load {{FILE}}

tui-seed SEED:
    cargo run -p balatro-tui -- --seed {{SEED}}

jkr *FILES:
    cargo run -p balatro-jkr --bin jkr -- {{FILES}}

profile *ARGS:
    cargo run -p balatro-profile --bin profile -- {{ARGS}}
