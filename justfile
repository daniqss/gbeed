default: run

mod memoria 'docs/memoria'
mod web 'frontends/debugger/static/web.just'
mod cross 'cross/cross.just'

build *ARGS:
    cargo build --features "${DISPLAY_FEATURES}" {{ARGS}}

run *ARGS:
    cargo run --features "${DISPLAY_FEATURES}" {{ARGS}}

check *ARGS:
    cargo check --features "${DISPLAY_FEATURES}" {{ARGS}}

lint *ARGS:
    cargo fmt --all
    cargo clippy --workspace --all-targets --features "${DISPLAY_FEATURES}" {{ARGS}} -- -D warnings

test *ARGS: fetch-test-roms
    cargo test --features "${DISPLAY_FEATURES}" {{ARGS}}

clean:
    cargo clean
    just web clean
    just memoria clean

fetch-test-roms:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -d gb-test-roms ]; then
        git clone --depth 1 https://github.com/retrio/gb-test-roms.git
    fi
    if [ ! -d mts-20240926-1737-443f6e1 ]; then
        curl -fLO https://gekkio.fi/files/mooneye-test-suite/mts-20240926-1737-443f6e1/mts-20240926-1737-443f6e1.tar.xz
        tar -xJf mts-20240926-1737-443f6e1.tar.xz
        rm -f mts-20240926-1737-443f6e1.tar.xz
    fi
    if [ ! -f dmg_boot.bin ]; then
        curl -fL https://github.com/alloncm/MagenBoot/releases/download/0.2.0/dmg_boot.bin -o dmg_boot.bin
    fi

flamegraph *ARGS:
    RUSTFLAGS="-Cforce-frame-pointers=yes -Cforce-unwind-tables=yes" cargo flamegraph --profile bench --features "${DISPLAY_FEATURES}" -p gbeed-console {{ARGS}}
