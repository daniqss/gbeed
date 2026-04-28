{
  pkgs,
  commonPackages,
  outputs,
  system,
}:
pkgs.mkShell {
  # inherit console package inputs
  # we cannot do this with the other shells because of wayland/x11 dependencies
  inputsFrom = [outputs.packages.${system}.console];
  packages = commonPackages;

  env = {
    DISPLAY_FEATURES = pkgs.lib.concatStringsSep " " [
      "raylib/drm"
      "raylib/opengl_es_20"
    ];

    RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    NIX_CFLAGS_COMPILE = "-I${pkgs.libdrm.dev}/include/libdrm";

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
      libdrm
      libgbm
      mesa
      libGL
      libglvnd
    ]);
  };
}
