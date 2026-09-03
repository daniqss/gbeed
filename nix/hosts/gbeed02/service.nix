{
  username,
  config,
  pkgs,
  lib,
  ...
}: let
  gbeed = pkgs.gbeed.console-gamepi13;
  user = config.users.users.${username};
in {
  # change rom dirs to user
  systemd.tmpfiles.rules = [
    "d ${user.home}/roms 0755 ${user.name} ${user.group} -"
    "d ${user.home}/saves 0755 ${user.name} ${user.group} -"
  ];

  # quiet the boot, the emulator should be the first thing the user sees, not the kernel log
  boot.consoleLogLevel = 0;
  boot.kernelParams = ["quiet" "systemd.show_status=false"];
  systemd.targets.getty.wants = lib.mkForce [];

  systemd.services.gbeed = {
    description = "Game Boy Emulator for Embedded Devices";
    after = ["multi-user.target"];
    wantedBy = ["multi-user.target"];

    onSuccess = ["getty@tty1.service"];
    onFailure = ["getty@tty1.service"];

    environment.HOME = user.home;

    startLimitIntervalSec = 60;
    startLimitBurst = 3;

    serviceConfig = {
      Type = "simple";
      User = user.name;
      Group = "users";
      WorkingDirectory = user.home;

      ExecStart = lib.getExe gbeed;

      # needs gpio to read the buttons and drm to reach the panel
      SupplementaryGroups = ["video" "render" "input" "gpio"];

      # quitting from the menu is a clean exit and must stay quit, only crashes come back
      Restart = "on-failure";
      RestartSec = "3";
    };
  };

  environment.systemPackages = [gbeed];
}
