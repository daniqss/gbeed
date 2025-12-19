# gbeed
WIP DMG Game Boy emulator for embedded devices. This project aims to provide a simple DMG Game Boy emulator that can run both over a graphical session and in a DRM/KMS environment, in a normal x86-64 Linux pc or in a Raspberry Pi Zero running Linux. To allow this it'll use SDL2 as a library to deal with graphics, input and audio.


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
# TODO: test this
# rust toolchain, recommended way is using rustup to manage rust versions
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# raylib dependencies
sudo apt install build-essential git libasound2-dev libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev libglu1-mesa-dev libxcursor-dev libxinerama-dev libxkbcommon-dev libdrm-dev libgbm-dev
```

To build and run the project without just, in x11 you can just `cargo run -- <game_rom> <boot_rom>`, but in other environments you will need the to add the corresponding features, as are used in `justfile`.

### Display environments
The flake exposes three different shells for different display environments. Direnv will check some environment variables to automatically select the right shell, but you can also manually select the shell this way:
- `x11`: `nix develop .#x11`
- `wayland`: `nix develop .#wayland`
- `drm`: `nix develop .#drm`

The default shell is `x11`, because will at least run in wayland sessions too using xwayland

### How to run tests
To run the tests, you can use just:
```sh
just test
```

The boot rom test needs a valid dmg boot rom file to run in the project root, named `dmg_boot.bin` 
