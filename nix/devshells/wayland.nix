{
  pkgs,
  commonPackages,
  commonEnvs,
  commonShellHook,
  platformPackages,
  platformFeatures,
}:
pkgs.mkShell {
  packages = commonPackages;
  buildInputs = platformPackages;
  shellHook = commonShellHook;

  env =
    commonEnvs
    // {
      DISPLAY_FEATURES = pkgs.lib.concatStringsSep " " platformFeatures;
      RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath platformPackages;
    };
}
