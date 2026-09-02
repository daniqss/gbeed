{
  hostname,
  pkgs,
  ...
}: let
  wifiNetworks = {
    "miwifi" = "contrasenha";
  };

  iwdCredentials =
    pkgs.lib.mapAttrsToList (
      ssid: psk: let
        file = pkgs.writeText "${ssid}.psk" ''
          [Security]
          Passphrase=${psk}

          [Settings]
          AutoConnect=true
        '';
      in "C+ /var/lib/iwd/${ssid}.psk 0600 root root - ${file}"
    )
    wifiNetworks;
in {
  # ssh
  services.openssh = {
    enable = true;
    settings.PermitRootLogin = "yes";
  };

  networking = {
    hostName = hostname;

    useNetworkd = true;
    useDHCP = true;

    wireless = {
      enable = false;

      iwd = {
        enable = true;
        settings = {
          Network.EnableIPv6 = true;
          Settings.AutoConnect = true;
        };
      };
    };
  };

  systemd = {
    tmpfiles.rules = ["d /var/lib/iwd 0700 root root -"] ++ iwdCredentials;

    # nothing is plugged in on boot, don't wait two minutes for a link
    network.wait-online.enable = false;
  };

  # publish gbeed02.local so the console can be reached without knowing its ip
  services.avahi = {
    enable = true;
    nssmdns4 = true;
    publish = {
      enable = true;
      addresses = true;
      workstation = true;
    };
  };

  environment.systemPackages = with pkgs; [
    iw
    util-linux
  ];
}
