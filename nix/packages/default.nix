{pkgs, ...}: {
  console = pkgs.callPackage ./console.nix {};
  debugger = pkgs.callPackage ./debugger.nix {};
  default = pkgs.callPackage ./console.nix {};
}
