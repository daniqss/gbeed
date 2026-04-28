{
  inputs,
  outputs,
}: {
  # build a NixOS configuration for SD Images system for a Raspberry Pi Zero 2 host
  mkGbeedSystem = {
    host,
    modules ? [],
    ...
  }:
    inputs.nixos-raspberrypi.lib.nixosSystem {
      specialArgs = {inherit inputs outputs;};
      modules =
        [
          inputs.nixos-raspberrypi.nixosModules.sd-image
          host
        ]
        ++ modules;
    };

  # platform-specific packages and cargo features for raylib display backends
  x11Packages = pkgs:
    with pkgs; [
      libGL
      xorg.libX11
      xorg.libXrandr
      xorg.libXinerama
      xorg.libXcursor
      xorg.libXi
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
