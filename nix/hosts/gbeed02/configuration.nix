{
  hostname,
  username,
  config,
  pkgs,
  lib,
  ...
}: {
  imports = [
    ./service.nix
  ];

  image.baseName = lib.mkForce hostname;

  system.stateVersion = config.system.nixos.release;
  time.timeZone = "UTC";
  networking.hostName = hostname;

  users.users.${username} = {
    isNormalUser = true;
    extraGroups = ["video" "render" "input" "gpio" "wheel"];
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
    pkgs.git
    pkgs.tree
    pkgs.htop

    pkgs.libdrm
    pkgs.kmscube
    pkgs.evtest

    # not bring innecessary things like gtk and sdl
    (pkgs.alsa-utils.override {
      withPipewireLib = false;
      alsa-plugins = pkgs.alsa-plugins.override {ffmpeg = null;};
    })
  ];
}
