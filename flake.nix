{
  description = "DMG Game Boy Emulator for embedded devices";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = {
    nixpkgs,
    fenix,
    ...
  }: let
    eachSystem = f:
      nixpkgs.lib.genAttrs ["x86_64-linux" "aarch64-linux"]
      (system:
        f system (import nixpkgs {
          inherit system;
          overlays = [fenix.overlays.default];
        }));
  in {
    devShells = eachSystem (system: pkgs: {
      default = pkgs.mkShell {
        buildInputs = [
          pkgs.cargo
          pkgs.cargo-expand
          pkgs.rust-analyzer
          pkgs.rustc
          pkgs.clippy
          fenix.packages.${system}.latest.rustfmt

          # raylib dependencies
          pkgs.glfw-wayland
          pkgs.cmake
          pkgs.clang
          pkgs.wayland
        ];

        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        LD_LIBRARY_PATH = with pkgs;
          lib.makeLibraryPath [
            libGL
          ];
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
      };
    });
  };
}
