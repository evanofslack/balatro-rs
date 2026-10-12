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

tui-record FILE:
    cargo run -p balatro-tui -- --record {{FILE}}

tui-record-default:
    cargo run -p balatro-tui -- --record

tui-replay FILE:
    cargo run -p balatro-tui -- --replay {{FILE}}

tui-library:
    cargo run -p balatro-tui -- --library

jkr *FILES:
    cargo run -p balatro-jkr --bin jkr -- {{FILES}}

profile *ARGS:
    cargo run -p balatro-profile --bin profile -- {{ARGS}}

seed *ARGS:
    cargo run -p balatro-seed --bin explore -- {{ARGS}}

calc *ARGS:
    cargo run -p balatro-rs --bin calc -- {{ARGS}}
