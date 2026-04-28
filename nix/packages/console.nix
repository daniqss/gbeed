{
  lib,
  rustPlatform,
  cmake,
  clang,
  pkg-config,
  libdrm,
  mesa,
  libGL,
  libglvnd,
  alsa-lib,
}: let
  name = "gbeed-console";
  version = "0.1.0";
  description = "DMG Game Boy Emulator for embedded devices - Console Frontend";
  repository = "https://github.com/daniqss/gbeed";
in
  rustPlatform.buildRustPackage {
    pname = name;
    inherit version;

    src = lib.cleanSource ../..;
    cargoLock = {
      lockFile = ../../Cargo.lock;
      allowBuiltinFetchGit = true;
    };

    nativeBuildInputs = [
      cmake
      clang
      pkg-config
      rustPlatform.bindgenHook
    ];

    buildInputs = [
      libdrm
      mesa
      libGL
      libglvnd
      alsa-lib
    ];

    buildFeatures = [
      "raylib/drm"
      "raylib/opengl_es_20"
    ];

    # Only build the console frontend
    cargoBuildFlags = ["-p" "gbeed-console"];

    env = {
      NIX_CFLAGS_COMPILE = "-I${libdrm.dev}/include/libdrm";
    };

    meta = with lib; {
      mainProgram = "gbeed-console";
      inherit description;
      homepage = repository;
      license = licenses.gpl2;
      platforms = platforms.linux;
    };
  }
