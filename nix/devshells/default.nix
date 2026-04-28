{
  inputs,
  outputs,
  system,
  pkgs,
}: let
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
  x11 = pkgs.callPackage ./x11.nix {inherit commonPackages;};
  wayland = pkgs.callPackage ./wayland.nix {inherit commonPackages;};
  drm = pkgs.callPackage ./drm.nix {inherit commonPackages outputs system;};
  wasm = pkgs.callPackage ./wasm.nix {inherit rustToolchain;};
  default = pkgs.callPackage ./x11.nix {inherit commonPackages;};
}
