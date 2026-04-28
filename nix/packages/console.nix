{
  lib,
  rustPlatform,
  cmake,
  clang,
  pkg-config,
  libdrm,
  drmPackages,
  drmFeatures,
  ...
}: let
  inherit ((lib.importTOML ../../frontends/console/Cargo.toml).package) name version description repository;
in
  rustPlatform.buildRustPackage {
    pname = name;
    inherit version;

    src = lib.cleanSource ../..;
    cargoLock = {
      lockFile = ../../Cargo.lock;
      allowBuiltinFetchGit = true;
    };

    nativeBuildInputs = [cmake clang pkg-config rustPlatform.bindgenHook];
    buildInputs = drmPackages;
    buildFeatures = drmFeatures;
    cargoBuildFlags = ["-p" name];

    env.NIX_CFLAGS_COMPILE = "-I${libdrm.dev}/include/libdrm";

    meta = with lib; {
      inherit description;
      homepage = repository;
      mainProgram = "gbeed";
      license = licenses.gpl2;
      platforms = platforms.linux;
    };
  }
