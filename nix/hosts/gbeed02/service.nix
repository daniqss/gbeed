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
  systemd.tmpfiles.rules = [
    "d ${user.home}/roms 0755 ${user.name} ${user.group} -"
    "d ${user.home}/saves 0755 ${user.name} ${user.group} -"
  ];

  systemd.services.gbeed = {
    description = "Game Boy Emulator for Embedded Devices";
    after = ["multi-user.target"];
    wantedBy = ["multi-user.target"];

    environment.HOME = user.home;

    startLimitIntervalSec = 0;

    serviceConfig = {
      Type = "simple";
      User = user.name;
      Group = "users";
      WorkingDirectory = user.home;

      ExecStart = lib.getExe gbeed;

      # needs gpio to read the buttons and drm to reach the panel
      SupplementaryGroups = ["video" "render" "input" "gpio"];

      Restart = "always";
      RestartSec = "3";
    };
  };

  environment.systemPackages = [gbeed];
}
