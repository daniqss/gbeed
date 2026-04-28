{
  pkgs,
  commonPackages,
}: let
  waylandPackages = with pkgs; [
    wayland
    libxkbcommon
    alsa-lib
    glfw
  ];
in
  pkgs.mkShell {
    packages = commonPackages;
    buildInputs = waylandPackages;

    env = {
      DISPLAY_FEATURES = pkgs.lib.concatStringsSep " " [
        "raylib/wayland"
        "raylib/USE_EXTERNAL_GLFW"
      ];

      RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath waylandPackages;
    };
  }
