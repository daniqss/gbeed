set working-directory := '../../..'

default: build

build:
    RUSTFLAGS="-C panic=unwind" cargo build --target wasm32-unknown-emscripten -p gbeed-debugger --release
    mkdir -p dist
    cp target/wasm32-unknown-emscripten/release/gbeed_debugger.wasm dist/
    cp target/wasm32-unknown-emscripten/release/gbeed-debugger.js dist/
    cp -r frontends/debugger/static/* dist/

run:
    cargo build --target wasm32-unknown-emscripten -p gbeed-debugger
    mkdir -p dist
    cp target/wasm32-unknown-emscripten/debug/gbeed_debugger.wasm dist/
    cp target/wasm32-unknown-emscripten/debug/gbeed-debugger.js dist/
    cp -r frontends/debugger/static/* dist/
    python3 -m http.server 8080 --directory dist --bind 0.0.0.0

clean:
    rm -rf dist
