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
    rev = "59bab283e088da53e474e2b6f4e4903b2c073533";
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  };

  nativeBuildInputs = [cmake];
  buildInputs = [libraspberrypi];

  cmakeFlags = [
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
