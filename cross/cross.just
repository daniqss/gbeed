set working-directory := '..'

default:
    @just --list cross

alpine:
    sudo podman run --rm --privileged docker.io/tonistiigi/binfmt --install arm
    podman build --platform linux/arm/v6 -f Dockerfile.cross.alpine -t gbeed-armv6l .
    podman create --name gbeed-armv6l-tmp gbeed-armv6l
    podman cp gbeed-armv6l-tmp:/app/target/release/gbeed ./gbeed-alpine
    podman rm gbeed-armv6l-tmp
    @echo "Release binary for armv6l (DRM/KMS backend, Alpine Linux) generated at ./gbeed-alpine"

debian:
    podman build -f Dockerfile.cross.debian -t gbeed-armv6l-debian .
    podman create --name gbeed-armv6l-debian-tmp gbeed-armv6l-debian
    podman cp gbeed-armv6l-debian-tmp:/app/target/arm-unknown-linux-gnueabihf/release/gbeed ./gbeed-debian
    podman rm gbeed-armv6l-debian-tmp
    @echo "Release binary for armv6l (X11/GLFW backend, Debian Bookworm) generated at ./gbeed-debian"
