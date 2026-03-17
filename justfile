default: run

build *ARGS:
    cargo build --features "${DISPLAY_FEATURES}" {{ARGS}}

run *ARGS:
    cargo run --features "${DISPLAY_FEATURES}" {{ARGS}}

check *ARGS:
    cargo check --features "${DISPLAY_FEATURES}" {{ARGS}}

test *ARGS:
    cargo test --features "${DISPLAY_FEATURES}" {{ARGS}}


crossbuild:
    sudo podman run --rm --privileged docker.io/tonistiigi/binfmt --install arm
    podman build --platform linux/arm/v6 -f Dockerfile.cross -t gbeed-armv6l .
    podman create --name gbeed-armv6l-tmp gbeed-armv6l
    podman cp gbeed-armv6l-tmp:/app/target/release/gbeed ./gbeed
    podman rm gbeed-armv6l-tmp
    @echo "Release binary for armv6l generated at ./gbeed"
