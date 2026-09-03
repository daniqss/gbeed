{
  hostname,
  username,
  config,
  pkgs,
  lib,
  ...
}: {
  imports = [
    # ./network.nix
    ./sd-image.nix
    ./service.nix
  ];

  image.baseName = lib.mkForce hostname;
  system.stateVersion = config.system.nixos.release;

  time.timeZone = "UTC";

  users.users.${username} = {
    isNormalUser = true;
    extraGroups = ["wheel" "networkmanager" "video" "render" "input" "gpio"];

    initialPassword = hostname;
    home = "/home/${username}";
  };
  users.users.root.initialPassword = hostname;

  security.sudo = {
    enable = true;
    wheelNeedsPassword = false;
  };

  services.getty.autologinUser = username;

  environment.systemPackages = with pkgs; [
    tree
    htop
    vim
  ];
}
