{
  inputs,
  outputs,
  system,
  pkgs,
}: let
  inherit (outputs) lib;

  rustToolchain = with inputs.fenix.packages.${system};
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
  x11 = pkgs.callPackage ./x11.nix {
    inherit commonPackages;
    platformPackages = lib.x11Packages pkgs;
    platformFeatures = lib.x11Features;
  };
  wayland = pkgs.callPackage ./wayland.nix {
    inherit commonPackages;
    platformPackages = lib.waylandPackages pkgs;
    platformFeatures = lib.waylandFeatures;
  };
  drm = pkgs.callPackage ./drm.nix {
    inherit commonPackages outputs system;
    platformFeatures = lib.drmFeatures;
    platformPackages = lib.drmPackages pkgs;
  };
  wasm = pkgs.callPackage ./wasm.nix {inherit rustToolchain;};
  default = pkgs.callPackage ./x11.nix {
    inherit commonPackages;
    platformPackages = lib.x11Packages pkgs;
    platformFeatures = lib.x11Features;
  };
}
