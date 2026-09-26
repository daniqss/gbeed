{
  pkgs,
  commonPackages,
  platformPackages,
  platformFeatures,
}:
pkgs.mkShell {
  name = "wayland";
  packages = commonPackages;
  buildInputs = platformPackages;

  env = {
    DISPLAY_FEATURES = pkgs.lib.concatStringsSep " " platformFeatures;
    RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath platformPackages;
  };
}
