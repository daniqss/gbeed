{
  pkgs,
  commonPackages,
  platformPackages,
  platformFeatures,
  outputs,
  system,
}:
pkgs.mkShell {
  inputsFrom = [outputs.packages.${system}.console];
  packages = commonPackages;

  env = {
    DISPLAY_FEATURES = pkgs.lib.concatStringsSep " " platformFeatures;
    RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    NIX_CFLAGS_COMPILE = "-I${pkgs.libdrm.dev}/include/libdrm";
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath platformPackages;
  };
}
