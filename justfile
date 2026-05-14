default: run

build *ARGS:
    cargo build --features "${DISPLAY_FEATURES}" {{ARGS}}

run *ARGS:
    cargo run --features "${DISPLAY_FEATURES}" {{ARGS}}

check *ARGS:
    cargo check --features "${DISPLAY_FEATURES}" {{ARGS}}

test *ARGS:
    cargo test --features "${DISPLAY_FEATURES}" {{ARGS}}

flamegraph *ARGS:
    cargo flamegraph --profile bench --features "${DISPLAY_FEATURES}" -p gbeed-console {{ARGS}}

web-build:
    RUSTFLAGS="-C panic=unwind" cargo build --target wasm32-unknown-emscripten -p gbeed-debugger --release
    mkdir -p dist
    cp target/wasm32-unknown-emscripten/release/gbeed_debugger.wasm dist/
    cp target/wasm32-unknown-emscripten/release/gbeed-debugger.js dist/
    cp -r frontends/debugger/static/* dist/

web-run:
    cargo build --target wasm32-unknown-emscripten -p gbeed-debugger
    mkdir -p dist
    cp target/wasm32-unknown-emscripten/debug/gbeed_debugger.wasm dist/
    cp target/wasm32-unknown-emscripten/debug/gbeed-debugger.js dist/
    cp -r frontends/debugger/static/* dist/
    python3 -m http.server 8080 --directory dist --bind 0.0.0.0

cross-build-alpine:
    sudo podman run --rm --privileged docker.io/tonistiigi/binfmt --install arm
    podman build --platform linux/arm/v6 -f Dockerfile.cross -t gbeed-armv6l .
    podman create --name gbeed-armv6l-tmp gbeed-armv6l
    podman cp gbeed-armv6l-tmp:/app/target/release/gbeed ./gbeed
    podman rm gbeed-armv6l-tmp
    @echo "Release binary for armv6l (Alpine Linux) generated at ./gbeed"

cross-build-debian:
    podman build -f Dockerfile.cross.debian -t gbeed-armv6l-debian .
    podman create --name gbeed-armv6l-debian-tmp gbeed-armv6l-debian
    podman cp gbeed-armv6l-debian-tmp:/app/target/arm-unknown-linux-gnueabihf/release/gbeed ./gbeed
    podman rm gbeed-armv6l-debian-tmp
    @echo "Release binary for armv6l (Debian Bookworm) generated at ./gbeed"
