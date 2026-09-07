{inputs, ...}: {
  # creates firmware blob encoders for SPI panels driven by `panel-mipi-dbi`
  mipi-dbi = import ./mipi-dbi.nix {inherit (inputs.nixpkgs) lib;};

  rustSource = let
    inherit (inputs.nixpkgs) lib;
  in
    lib.fileset.toSource {
      root = ../..;
      fileset = lib.fileset.unions [
        ../../Cargo.toml
        ../../Cargo.lock
        ../../core
        ../../frontends
      ];
    };

  # platform-specific packages and cargo features for raylib display backends
  x11Packages = pkgs:
    with pkgs; [
      libGL
      libx11
      libxrandr
      libxinerama
      libxcursor
      libxi
      alsa-lib
    ];
  x11Features = [];

  waylandPackages = pkgs:
    with pkgs; [
      wayland
      libxkbcommon
      alsa-lib
      glfw
    ];
  waylandFeatures = ["raylib/wayland" "raylib/USE_EXTERNAL_GLFW"];

  drmPackages = pkgs:
    with pkgs; [
      libdrm
      libgbm
      mesa
      libGL
      libglvnd
      alsa-lib
    ];
  drmFeatures = ["raylib/drm" "raylib/opengl_es_20"];
}
