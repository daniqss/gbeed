{
  pkgs,
  commonPackages,
}: let
  x11Packages = with pkgs; [
    libGL
    xorg.libX11
    xorg.libXrandr
    xorg.libXinerama
    xorg.libXcursor
    xorg.libXi
    alsa-lib
  ];
in
  pkgs.mkShell {
    packages = commonPackages;
    buildInputs = x11Packages;

    env = {
      DISPLAY_FEATURES = "";
      RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath x11Packages;
    };
  }
