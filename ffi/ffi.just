set working-directory := '../'

# builds libgbeed.so and libgbeed.a in target/release
build *ARGS:
    cargo build -p gbeed-ffi --release {{ARGS}}

# builds a no_std and heapless libgbeed.a for an embedded target, which already defines `panic=abort`
build-nostd TARGET *ARGS:
    cargo build -p gbeed-ffi --release --no-default-features --target {{TARGET}} {{ARGS}}

# regenerates ffi/include/gbeed.h from gbeed-core, needs a nightly toolchain to expand the macros
header:
    cbindgen -q --config ffi/cbindgen.toml --crate gbeed-core -o ffi/include/gbeed.h core

# fails if ffi/include/gbeed.h is outdated
header-check:
    cbindgen -q --config ffi/cbindgen.toml --crate gbeed-core --verify -o ffi/include/gbeed.h core

test *ARGS:
    cargo test -p gbeed-core --features ffi {{ARGS}}

# runs the ffi tests with the heapless core, as used by build-nostd
test-nostd *ARGS:
    cargo test -p gbeed-core --no-default-features --features ffi --lib ffi {{ARGS}}

example-c ROM *ARGS: build
    cc ffi/examples/c/main.c -Wall -Wextra -Iffi/include -Ltarget/release -lgbeed -o target/release/gbeed-c
    LD_LIBRARY_PATH=target/release target/release/gbeed-c "{{ROM}}" {{ARGS}}

example-python ROM *ARGS: build
    python3 ffi/examples/python/gbeed.py "{{ROM}}" {{ARGS}}
