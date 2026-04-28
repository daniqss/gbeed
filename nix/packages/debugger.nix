{
  lib,
  rustPlatform,
  cmake,
  clang,
  pkg-config,
  x11Packages,
  waylandPackages,
  x11Features,
  waylandFeatures,
  # toggle wayland: (packages.debugger.override {withWayland = true;})
  withWayland ? false,
  ...
}: let
  inherit ((lib.importTOML ../../frontends/debugger/Cargo.toml).package) name version description repository;
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
    buildInputs =
      if withWayland
      then waylandPackages
      else x11Packages;
    buildFeatures =
      if withWayland
      then waylandFeatures
      else x11Features;
    cargoBuildFlags = ["-p" name];

    meta = with lib; {
      inherit description;
      homepage = repository;
      mainProgram = name;
      license = licenses.gpl2;
      platforms = platforms.linux;
    };
  }
