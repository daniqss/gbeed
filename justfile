default: run

build *ARGS:
    cargo build --features "${DISPLAY_FEATURES}" {{ARGS}}

run *ARGS:
    cargo run --features "${DISPLAY_FEATURES}" {{ARGS}}

check *ARGS:
    cargo check --features "${DISPLAY_FEATURES}" {{ARGS}}

test *ARGS:
    cargo test --features "${DISPLAY_FEATURES}" {{ARGS}}

web-build *ARGS:
    RUSTFLAGS="-C link-arg=-lidbfs.js -C link-arg=-sFORCE_FILESYSTEM=1 -C link-arg=-sEXPORTED_RUNTIME_METHODS=['FS']" cargo build --target wasm32-unknown-emscripten -p gbeed-ui {{ARGS}}
    mkdir -p dist
    cp target/wasm32-unknown-emscripten/release/gbeed.wasm dist/
    cp target/wasm32-unknown-emscripten/release/gbeed.js dist/
    cp -r ui/static/* dist/


ip := `hostname -I | awk '{print $1}'`

web-run: web-build
    @echo "http://{{ip}}:8080"
    python3 -m http.server 8080 --directory dist --bind 0.0.0.0

cross-build:
    sudo podman run --rm --privileged docker.io/tonistiigi/binfmt --install arm
    podman build --platform linux/arm/v6 -f Dockerfile.cross -t gbeed-armv6l .
    podman create --name gbeed-armv6l-tmp gbeed-armv6l
    podman cp gbeed-armv6l-tmp:/app/target/release/gbeed ./gbeed
    podman rm gbeed-armv6l-tmp
    @echo "Release binary for armv6l generated at ./gbeed"
