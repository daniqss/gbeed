{
  lib,
  pkgs,
}: {
  # ST7789V initialisation blob the kernel loads to bring the panel up
  gamepi13-panel = lib.mipi-dbi.mkFirmware pkgs (import ./panel.nix);

  # PWM audio on GPIO18 alone
  gamepi13-audremap18 = pkgs.deviceTree.compileDTS {
    name = "gbeed-audremap18";
    dtsFile = pkgs.writeText "gbeed-audremap18.dts" (import ./audremap18.nix);
  };
}
