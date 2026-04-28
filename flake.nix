{
  description = "DMG Game Boy Emulator for embedded devices";

  nixConfig = {
    extra-substituters = [
      "https://nixos-raspberrypi.cachix.org"
    ];
    extra-trusted-public-keys = [
      "nixos-raspberrypi.cachix.org-1:4iMO9LXa8BqhU+Rpg6LQKiGa2lsNh/j2oiYLNOQ5sPI="
    ];
    connect-timeout = 5;
  };

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    fenix.url = "github:nix-community/fenix";
    nixos-raspberrypi.url = "github:nvmd/nixos-raspberrypi/main";
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    nixos-raspberrypi,
    ...
  }@inputs: let
    eachSystem = f:
      nixpkgs.lib.genAttrs ["x86_64-linux" "aarch64-linux"]
      (system:
        f system (import nixpkgs {
          inherit system;
          overlays = [fenix.overlays.default];
        }));
  in {
    devShells = eachSystem (system: pkgs: let
      rustToolchain = with fenix.packages.${system};
        combine [
          latest.cargo
          latest.rustc
          latest.clippy
          latest.rustfmt
          targets.wasm32-unknown-emscripten.latest.rust-std
        ];

      commonPackages = with pkgs; [
        just
        cmake
        clang

        rustToolchain
        cargo-flamegraph
        perf
      ];
    in {
      # shell for x11 environments
      x11 = let
        x11Packages = with pkgs; [
          libGL
          xorg.libX11
          xorg.libXrandr
          xorg.libXinerama
          xorg.libXcursor
          xorg.libXi
          alsa-lib
        ];
        x11Features = "";
      in
        pkgs.mkShell {
          buildInputs = commonPackages ++ x11Packages;

          env = {
            DISPLAY_FEATURES = x11Features;
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

            LD_LIBRARY_PATH = with pkgs;
              lib.makeLibraryPath x11Packages;
          };
        };

      # shell for wayland environments
      wayland = let
        waylandPackages = with pkgs; [
          wayland
          libxkbcommon
          alsa-lib
          glfw
        ];
        waylandFeatures = [
          "raylib/wayland"
          "raylib/USE_EXTERNAL_GLFW"
        ];
      in
        pkgs.mkShell {
          buildInputs = commonPackages ++ waylandPackages;

          env = {
            DISPLAY_FEATURES = pkgs.lib.concatStringsSep " " waylandFeatures;
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

            LD_LIBRARY_PATH = with pkgs;
              lib.makeLibraryPath waylandPackages;
          };
        };

      # shell for drm environments
      drm = let
        drmPackages = with pkgs; [
          pkg-config
          libdrm
          libgbm
          mesa
          libGL
          libglvnd
        ];
        drmFeatures = [
          "raylib/drm"
          "raylib/opengl_es_20"
        ];
      in
        pkgs.mkShell {
          buildInputs = commonPackages ++ drmPackages;

          env = {
            DISPLAY_FEATURES = pkgs.lib.concatStringsSep " " drmFeatures;
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

            NIX_CFLAGS_COMPILE = "-I${pkgs.libdrm.dev}/include/libdrm";

            LD_LIBRARY_PATH = with pkgs;
              lib.makeLibraryPath drmPackages;
          };
        };

      # used to target wasm
      wasm = pkgs.mkShell {
        buildInputs = with pkgs; [
          just
          cmake
          clang
          emscripten
          python3

          rustToolchain
        ];

        env = {
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          EMCC_CFLAGS = pkgs.lib.concatStringsSep " " [
            "-O3"
            "-sUSE_GLFW=3"
            "-sASSERTIONS=1"
            "-sWASM=1"
            "-sASYNCIFY"
            "-sGL_ENABLE_GET_PROC_ADDRESS=1"
            "-sEXPORTED_RUNTIME_METHODS=FS,HEAPU8,ccall,cwrap"
          ];
        };
      };

      # defaulting to x11 because wayland will use it over xwayland anyway
      default = self.devShells.${system}.x11;
    });

    # NixOS configuration for RPi Zero 2 running gbeed
    nixosConfigurations.gbeed02 = nixos-raspberrypi.lib.nixosSystemFull {
      specialArgs = inputs;
      modules = [
        nixos-raspberrypi.nixosModules.sd-image
        ./nix/gbeed02.nix
      ];
    };

    # SD card images (ready-to-use, not installers)
    installerImages.gbeed02 =
      self.nixosConfigurations.gbeed02.config.system.build.sdImage;
  };
}
