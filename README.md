# gbeed
WIP DMG Game Boy emulator for embedded devices. This project aims to provide a simple DMG Game Boy emulator that can run both over a graphical session and in a DRM/KMS environment, in a normal x86-64 Linux pc or in a Raspberry Pi Zero running Linux. To allow this it'd use SDL2 as a library to deal with graphics, input and audio.

## How to build
To build the project the Rust toolchain and the SDL2 libraries are required. They can be easily install using the Nix shell provided:
```sh
nix develop
cargo build
```

## How to run
The current program just displays the provided rom information on screen
```sh
cargo run -- <path-to-game-rom> <path-to-boot-rom>
```

To run the tests just run:
```sh
cargo test
```

The boot rom test needs a valid boot rom file in the project root named `dmg_boot.bin` to run properly.
