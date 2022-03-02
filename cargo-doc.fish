#!/usr/bin/env fish

# 1. Make sure to install cargo-watch via `cargo install cargo-watch`.
# More info about cargo-watch: https://crates.io/crates/cargo-watch

# 2. Make sure to install cargo-limit via `cargo install cargo-limit`.
# More info about cargo-limit: https://crates.io/crates/cargo-limit

# cargo doc --open
cargo watch -x check -x 'ldoc -q --color always' -c -q
