# to use gamepi13 without dispmax and fbcp, we need to use a firmware blob for the panel-mipi-dbi driver.
# this file generates the blob at build time, this blob consists of a header and a sequence of commands.
# the header is the ascii for "MIPI DBI" followed by seven null bytes and a version byte.
# the following are commands encoded as a sequence:
#   - of command bytes
#   - parameter count bytes
#   - the parameters themselves
{lib}: let
  hexOfInt = n:
    assert lib.assertMsg (n >= 0 && n <= 255) "mipi-dbi: byte out of range: ${toString n}";
      lib.fixedWidthString 2 "0" (lib.toLower (lib.toHexString n));

  hexByte = s:
    assert lib.assertMsg (builtins.match "[0-9a-fA-F]{2}" s != null) "mipi-dbi: not a hex byte: ${s}";
      lib.toLower s;

  # `{delay = ms;}` waits, `{cmd = "36"; params = ["78"];}` sends a DCS command
  encodeEntry = entry:
    if entry ? delay
    then ["00" "01" (hexOfInt entry.delay)]
    else let
      params = map hexByte (entry.params or []);
    in
      [(hexByte entry.cmd) (hexOfInt (builtins.length params))] ++ params;

  header = ["4d" "49" "50" "49" "20" "44" "42" "49" "00" "00" "00" "00" "00" "00" "00" "01"];
in rec {
  # the blob as a flat list of hex encoded bytes
  toBytes = commands: header ++ lib.concatMap encodeEntry commands;

  # the blob as a printf(1) escape string (like "\x4d\x49\x50\x49")
  toEscapes = commands: lib.concatMapStrings (b: "\\x${b}") (toBytes commands);

  # this package places the blob where `hardware.firmware` expects it.
  # the commands for the speficic panel are passed from `gamepi13/panel.nix`
  mkFirmware = pkgs: {
    name ? "panel",
    commands,
  }:
    pkgs.runCommand "mipi-dbi-firmware-${name}" {
      passthru = {inherit commands;};
    } ''
      mkdir -p $out/lib/firmware
      printf '%b' '${toEscapes commands}' > $out/lib/firmware/${name}.bin
    '';
}
