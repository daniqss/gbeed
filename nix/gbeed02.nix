{
  config,
  pkgs,
  lib,
  nixos-raspberrypi,
  ...
}: let
  gbeed = pkgs.callPackage ./gbeed-package.nix {};
in {
  imports = with nixos-raspberrypi.nixosModules; [
    raspberry-pi-02.base
  ];

  nixpkgs.hostPlatform = lib.mkForce "aarch64-linux";
  system.stateVersion = config.system.nixos.release;
  time.timeZone = "UTC";
  networking.hostName = "gbeed02";

  users.users.gbeed = {
    isNormalUser = true;
    extraGroups = ["video" "render" "input" "wheel"];
    initialHashedPassword = "gameboy";
    home = "/home/gbeed";
  };
  users.users.root.initialHashedPassword = "";
  security.sudo = {
    enable = true;
    wheelNeedsPassword = false;
  };

  services.getty.autologinUser = "gbeed";

  services.openssh = {
    enable = true;
    settings.PermitRootLogin = "yes";
  };
  networking.useNetworkd = true;
  networking.wireless.enable = false;
  networking.wireless.iwd = {
    enable = true;
    settings = {
      Network.EnableIPv6 = true;
      Settings.AutoConnect = true;
    };
  };
  networking.firewall.allowedUDPPorts = [5353];

  environment.systemPackages = [
    gbeed
    pkgs.tree
    pkgs.htop
  ];

  # gbeed systemd service, launches on boot
  systemd.services.gbeed = {
    description = "gbeed - Game Boy Emulator";
    after = ["multi-user.target"];
    wantedBy = ["multi-user.target"];

    environment = {
      HOME = "/home/gbeed";
    };

    serviceConfig = {
      Type = "simple";
      User = "gbeed";
      Group = "users";
      WorkingDirectory = "/home/gbeed";

      ExecStartPre = "${pkgs.coreutils}/bin/mkdir -p /home/gbeed/roms /home/gbeed/saves";
      ExecStart = "${gbeed}/bin/gbeed";

      Restart = "on-failure";
      RestartSec = "3";

      # DRM/KMS access
      SupplementaryGroups = ["video" "render" "input"];

      # TTY access for DRM
      TTYPath = "/dev/tty1";
      StandardInput = "tty";
      StandardOutput = "tty";
      StandardError = "journal";
      TTYVHangup = true;
      TTYReset = true;
    };
  };

  # filesystem layout for SD card
  fileSystems = {
    "/boot/firmware" = {
      device = "/dev/disk/by-label/FIRMWARE";
      fsType = "vfat";
      options = [
        "noatime"
        "noauto"
        "x-systemd.automount"
        "x-systemd.idle-timeout=1min"
      ];
    };
    "/" = {
      device = "/dev/disk/by-label/NIXOS_SD";
      fsType = "ext4";
      options = ["noatime"];
    };
  };

  system.nixos.tags = let
    cfg = config.boot.loader.raspberry-pi;
  in [
    "gbeed"
    "raspberry-pi-${cfg.variant}"
    cfg.bootloader
    config.boot.kernelPackages.kernel.version
  ];
}
