default: run

build *ARGS:
    cargo build --features "${DISPLAY_FEATURES}" {{ARGS}}

run *ARGS:
    cargo run --features "${DISPLAY_FEATURES}" {{ARGS}}

check *ARGS:
    cargo check --features "${DISPLAY_FEATURES}" {{ARGS}}

test *ARGS:
    cargo test --features "${DISPLAY_FEATURES}" {{ARGS}}
