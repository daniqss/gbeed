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

      ExecStartPre = "${pkgs.coreutils}/bin/mkdir -p ${user.home}/roms ${user.home}/saves";
      ExecStart = lib.getExe gbeed;

      # needs gpio to read the buttons and drm to reach the panel
      SupplementaryGroups = ["video" "render" "input" "gpio"];

      Restart = "always";
      RestartSec = "3";
    };
  };

  environment.systemPackages = [gbeed];
}
