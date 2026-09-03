{
  outputs,
  pkgs,
  ...
}: let
  inherit (outputs) lib;
  platformArgs = {
    drmPackages = lib.drmPackages pkgs;
    drmFeatures = lib.drmFeatures;
    x11Packages = lib.x11Packages pkgs;
    x11Features = lib.x11Features;
    waylandPackages = lib.waylandPackages pkgs;
    waylandFeatures = lib.waylandFeatures;

    inherit (lib) rustSource;
  };
in let
  console = pkgs.callPackage ./console.nix platformArgs;
  debugger = pkgs.callPackage ./debugger.nix platformArgs;
in {
  inherit console debugger;
  debugger-wayland = debugger.override {withWayland = true;};
  default = console;

  console-gamepi13 = console.override {withGamepi13 = true;};

  # panel initialisation blob for the gamepi13
  inherit (import ./gamepi13 {inherit lib pkgs;}) gamepi13-panel;
}
