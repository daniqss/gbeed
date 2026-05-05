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
  };
in let
  console = pkgs.callPackage ./console.nix platformArgs;
  debugger = pkgs.callPackage ./debugger.nix platformArgs;
  fbcp = pkgs.callPackage ./fbcp-ili9341.nix {};
in {
  inherit console debugger fbcp;
  debugger-wayland = debugger.override {withWayland = true;};
  default = console;
}
