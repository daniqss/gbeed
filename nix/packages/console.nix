{
  lib,
  rustPlatform,
  cmake,
  clang,
  pkg-config,
  libdrm,
  drmPackages,
  drmFeatures,
  rustSource,
  # build for the GamePi13: read the buttons from the GPIO header and render on
  # the SPI panel instead of on whichever DRM card raylib happens to open first
  withGamepi13 ? false,
  ...
}: let
  inherit ((lib.importTOML ../../frontends/console/Cargo.toml).package) name version description repository;

  # pass the panel in compile time, to avoid having to find the DRM cards at runtime
  panelDevice = "/dev/dri/gbeed-panel";
in
  rustPlatform.buildRustPackage {
    pname = name;
    inherit version;

    src = rustSource;
    cargoLock = {
      lockFile = ../../Cargo.lock;
      allowBuiltinFetchGit = true;
    };

    nativeBuildInputs = [cmake clang pkg-config rustPlatform.bindgenHook];
    buildInputs = drmPackages;
    buildFeatures = drmFeatures ++ lib.optional withGamepi13 "gamepi13";
    cargoBuildFlags = ["-p" name];

    # -Wno-error: raylib's vendored jar_mod.h triggers -Wstringop-overflow warnings
    env.NIX_CFLAGS_COMPILE = lib.concatStringsSep " " (
      [
        "-I${libdrm.dev}/include/libdrm"
        "-Wno-error"
      ]
      ++ lib.optional withGamepi13 ''-DDEFAULT_GRAPHIC_DEVICE_DRM="${panelDevice}"''
    );

    postFixup = ''
      patchelf --add-rpath ${lib.makeLibraryPath drmPackages} $out/bin/gbeed
    '';

    meta = with lib; {
      inherit description;
      homepage = repository;
      mainProgram = "gbeed";
      license = licenses.gpl2;
      platforms = platforms.linux;
    };
  }
