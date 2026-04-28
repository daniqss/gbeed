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
  name = "gbeed-debugger";
  version = "0.1.0";
  description = "DMG Game Boy Emulator for embedded devices - Debugger Frontend";
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

    # TODO: we need to specify x11 and wayland features
    cargoBuildFlags = ["-p" "gbeed-debugger"];

    meta = with lib; {
      mainProgram = "gbeed-debugger";
      inherit description;
      homepage = repository;
      license = licenses.gpl2;
      platforms = platforms.linux;
    };
  }
