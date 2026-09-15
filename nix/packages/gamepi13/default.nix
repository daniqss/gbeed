{
  lib,
  pkgs,
}: {
  # ST7789V initialisation blob the kernel loads to bring the panel up
  gamepi13-panel = lib.mipi-dbi.mkFirmware pkgs (import ./panel.nix);
}
