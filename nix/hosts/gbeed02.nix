{
  outputs,
  nixos-raspberrypi,
  config,
  pkgs,
  lib,
  ...
}: let
  hostname = "gbeed02";
  username = "gbeed";
  system = "aarch64-linux";
  gbeed = outputs.packages.${system}.console;
in {
  imports = with nixos-raspberrypi.nixosModules; [
    raspberry-pi-02.base
    raspberry-pi-02.display-vc4
  ];

  image.baseName = lib.mkForce hostname;

  hardware.graphics.enable = true;

  system.stateVersion = config.system.nixos.release;
  time.timeZone = "UTC";
  networking.hostName = hostname;


  users.users.gbeed = {
    isNormalUser = true;
    extraGroups = ["video" "render" "input" "wheel"];
    initialHashedPassword = hostname;
    home = "/home/${username}";
  };
  users.users.root.initialHashedPassword = hostname;

  security.sudo = {
    enable = true;
    wheelNeedsPassword = false;
  };

  services.getty.autologinUser = username;

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
    pkgs.git
    pkgs.tree
    pkgs.htop
  ];

  # gbeed systemd service, should launch on boot
  # systemd.services.gbeed = {
  #   description = "Game Boy Emulator for Embedded Devices";
  #   after = ["multi-user.target"];
  #   wantedBy = ["multi-user.target"];
  # 
  #   environment = {
  #     HOME = "/home/${username}";
  #   };
  # 
  #   serviceConfig = {
  #     Type = "simple";
  #     User = username;
  #     Group = "users";
  #     WorkingDirectory = "/home/${username}";
  # 
  #     ExecStartPre = "${pkgs.coreutils}/bin/mkdir -p /home/${username}/roms /home/${username}/saves";
  #     ExecStart = "gbeed";
  # 
  #     Restart = "on-failure";
  #     RestartSec = "3";
  # 
  #     # DRM/KMS access
  #     SupplementaryGroups = ["video" "render" "input"];
  # 
  #     # TTY access for DRM
  #     TTYPath = "/dev/tty1";
  #     StandardInput = "tty";
  #     StandardOutput = "tty";
  #     StandardError = "journal";
  #     TTYVHangup = true;
  #     TTYReset = true;
  #   };
  # };

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
