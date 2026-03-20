# gbeed
WIP DMG Game Boy emulator for embedded devices. This project aims to provide a simple DMG Game Boy emulator that can run both over a graphical session and in a DRM/KMS environment, in a normal x86-64 Linux pc or in a Raspberry Pi Zero running Linux. To allow this it'll use raylib as a library to deal with graphics, input and audio.

![gbeed](./assets/image.png)

## Status
### Games
Core emulator is mostly complete, and allow sufficintly good emulation in most games, besides some minor graphical glitches and deacceleration in some areas. The last remaining core feature that is not implemented yet is audio emulation.

The following games, the best-selling games of the DMG catalog, are tested in initial areas and are playable without major issues.

| Selling Ranking | Game               | Playable                 | Cartridge Type                        | ROM Size | RAM Size |
|-----------------|--------------------|--------------------------|---------------------------------------|----------|----------|
| 1               | Pokémon Red        | 🟩 Playable              | GB MBC3 + RAM + Battery               | 1024 KB  | 32 KB    |
| 2               | Tetris             | 🟩 Playable              | GB ROM Only                           | 32 KB    | None     |
| 3               | Pokémon Gold       | 🟩 Playable              | GBC MBC3 + Timer + RAM + Battery      | 2048 KB  | 32 KB    |
| 3               | Pokémon Crystal    | 🟩 Not Supported         | GBC Only MBC3 + Timer + RAM + Battery | 2048 KB  | 32 KB    |
| 4               | Super Mario Land   | 🟩 Playable              | GB MBC1                               | 64 KB    | None     |
| 5               | Super Mario Land 2 | 🟩 Playable<sup>*1</sup> | GB MBC1 + RAM + Battery               | 512 KB   | 8 KB     |
| 7               | Pokemon Pinball    | 🟩 Playable              | GBC MBC5 Rumble + RAM + Battery       | 1024 KB  | 8 KB     |
| 11              | Link's Awakining   | 🟩 Playable              | GB MBC1 + RAM + Battery               | 512 KB   | 8 KB     |

<sub>
<sup>1</sup> This game save file tends to get corrupted, crashing the game sometimes<br>
</sub>

### Tests
The emulator is tested using [Blargg's rom test](https://github.com/retrio/gb-test-roms) and [Mooneye test suite](https://github.com/Gekkio/mooneye-test-suite) and passes basic CPU instructions and MBC tests, but fails most of the timing tests. See passed tests in `core/tests`.

For PPU testing, gbeed passes [dmg-acid2](https://github.com/mattcurrie/dmg-acid2) test, so basic rendering is correct besides some minor issues and not fully accurate timing. This test must be run manually because it needs manual verification of the result.

## How to use
### Dependencies
This project uses `nix` flakes, and are the recommended way to manage dependencies. If installed and properly configured, using the project should be as easy as:
```sh
nix develop .
just run -- <game_rom> <boot_rom>   # just passes flags directly to cargo adding necessary features
```

If flakes are not enabled, you can use:
```sh
nix develop --experimental-features "nix-command flakes" .
```

If you have `direnv` installed and configured, just entering the project directory will automatically load the development environment after the first `direnv allow`
```sh
> direnv: error .envrc is blocked. Run `direnv allow` to approve its content
direnv allow
just run -- <game_rom> <boot_rom>
```

If you're not using nix, the dependencies must be installed manually, and according to the wanted environment. All dependencies are listed in the `flake.nix` file. For example, if you're using Debian in x11, you will need to install the rust toolchain with `rustc >= 1.91.1` and the raylib dependencies
```sh
# rust toolchain, recommended way is using rustup to manage rust versions
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# raylib dependencies
sudo apt install build-essential cmake clang git libasound2-dev libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev libglu1-mesa-dev libxcursor-dev libxinerama-dev libxkbcommon-dev libdrm-dev libgbm-dev
```

To build and run the project without just, in x11 you can just `cargo run -- <game_rom> <boot_rom>`, but in other environments you will need the to add the corresponding features, as are used in `justfile`.

### Display environments
The flake exposes three different shells for different display environments. Direnv will check some environment variables to automatically select the right shell, but you can also manually select the shell this way:
- `x11`: `nix develop .#x11`
- `wayland`: `nix develop .#wayland`
- `drm`: `nix develop .#drm`

The default shell is `x11`, because will at least run in wayland sessions too using xwayland.
DRM support works in both AMD and ARM GPU. In Nvidia both `opengl_es_20` and `opengl_es_30` segfault at init (`dic 20 13:12:41 stoneward kernel: gbeed[6765]: segfault at 0 ip 00007fa29f9b4d31 sp 00007fff9c2052d0 error 4 in libnvidia-egl-gbm.so.1.1.2[1d31,7fa29f9b4000+3000] likely on CPU 0 (core 0, socket 0)`).

### How to build for Alpine Linux in armv6l
#### Building in the Raspberry Pi Zero
1. Fresh Alpine Linux installation in the Raspberry Pi Zero SD card.
2. Install the dependencies:
```sh
doas apk add git cargo build-base cmake clang clang-dev pkgconf alsa-lib-dev libdrm-dev mesa-dev
```

3. Clone the repository and enter the project directory:
```sh
git clone https://github.com/daniqss/gbeed
cd gbeed
```

4. Treak the linker
```sh
touch im-libglvnd-fr-fr.c
cc -shared -o /tmp/libGLdispatch.so /tmp/im-libglvnd-fr-fr.c
```

5. Build the project:
```sh
export LIBCLANG_PATH=/usr/lib
RUSTFLAGS="-L /tmp" cargo build --features "raylib/drm raylib/opengl_es_20"
```

> [!WARNING]
> This is not an optimal way to build the project, as it will take a very long time. Also, the emulator's frontend is not yet adapted for this platform.

#### Cross-compilation in x86-64/aarch64
The easiest way to build the project for armv6l is through a podman or docker container using the provided `Dockerfile.cross`. This provides a fully isolated build environment.

Alternatively, you can use the following command to automate the process:
```sh
just crossbuild
```

This will:
1. Install the `arm` binfmt if needed.
2. Build the project inside an `arm32v6/alpine` container.
3. Extract the resulting binary as `./gbeed-armv6l`.

Or manually:
```sh
sudo podman run --rm --privileged docker.io/tonistiigi/binfmt --install arm
podman build --platform linux/arm/v6 -f Dockerfile.cross -t gbeed-armv6l .
podman create --name gbeed-armv6l-tmp gbeed-armv6l
podman cp gbeed-armv6l-tmp:/app/target/release/gbeed ./gbeed
podman rm gbeed-armv6l-tmp
```


### How to run tests
To run the tests, you can use just:
```sh
just test
```

The boot rom test needs a valid dmg boot rom file to run in the project root, named `dmg_boot.bin` 
