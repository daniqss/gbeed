{
  description = "DMG Game Boy Emulator for embedded devices";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = {
    self,
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
    devShells = eachSystem (system: pkgs: let
      commonPackages = with pkgs; [
        just

        cargo
        cargo-expand
        rust-analyzer
        rustc
        clippy
        fenix.packages.${system}.latest.rustfmt

        cmake
        clang
        glfw
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

          DISPLAY_FEATURES = x11Features;
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          LD_LIBRARY_PATH = with pkgs;
            lib.makeLibraryPath x11Packages;
        };

      # shell for wayland environments
      wayland = let
        waylandPackages = with pkgs; [
          wayland
          libxkbcommon
          alsa-lib
        ];
        waylandFeatures = [
          "raylib/wayland"
          "raylib/USE_EXTERNAL_GLFW"
        ];
      in
        pkgs.mkShell {
          buildInputs = commonPackages ++ waylandPackages;

          DISPLAY_FEATURES = pkgs.lib.concatStringsSep " " waylandFeatures;
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          LD_LIBRARY_PATH = with pkgs;
            lib.makeLibraryPath waylandPackages;
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

          DISPLAY_FEATURES = pkgs.lib.concatStringsSep " " drmFeatures;
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          NIX_CFLAGS_COMPILE = "-I${pkgs.libdrm.dev}/include/libdrm";

          LD_LIBRARY_PATH = with pkgs;
            lib.makeLibraryPath [
              alsa-lib
            ];
        };

      # defaulting to x11 because wayland will use it over xwayland anyway
      default = self.devShells.${system}.x11;
    });
  };
}
