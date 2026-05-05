{
  lib,
  stdenv,
  fetchFromGitHub,
  cmake,
  libraspberrypi,
}:
stdenv.mkDerivation rec {
  pname = "fbcp-ili9341";
  version = "unstable-2023-05-07";

  src = fetchFromGitHub {
    owner = "juj";
    repo = "fbcp-ili9341";
    rev = "d0ebacf7c1f30b19b50997ebb67ba4f70ab95368";
    hash = "sha256-53+HoVaVAfH7Rx6uMhQuELodK4zrDDwTmy6PpiOCtzU=";
  };

  nativeBuildInputs = [cmake];
  buildInputs = [libraspberrypi];

  # The project targets 32-bit ARM and adds flags invalid on aarch64
  postPatch = ''
    substituteInPlace CMakeLists.txt \
       --replace-fail "-marm" "" \
       --replace-fail "-mhard-float" "" \
       --replace-fail "-mfloat-abi=hard" "" \
       --replace-fail "-mtls-dialect=gnu2" "" \
       --replace-fail "-mabi=aapcs-linux" ""
  '';

  cmakeFlags = [
    "-DCMAKE_POLICY_VERSION_MINIMUM=3.5"
    "-DST7789VW=ON"
    "-DSPI_BUS_CLOCK_DIVISOR=6"
    "-DGPIO_TFT_DATA_CONTROL=25"
    "-DGPIO_TFT_RESET_PIN=27"
    "-DGPIO_TFT_BACKLIGHT=18"
    "-DSTATISTICS=0"
  ];

  installPhase = ''
    runHook preInstall
    install -Dm755 fbcp-ili9341 $out/bin/fbcp-ili9341
    runHook postInstall
  '';

  meta = with lib; {
    description = "Display driver for SPI-based LCD displays (ST7789VW)";
    homepage = "https://github.com/juj/fbcp-ili9341";
    license = licenses.mit;
    platforms = ["aarch64-linux" "armv7l-linux" "armv6l-linux"];
  };
}
