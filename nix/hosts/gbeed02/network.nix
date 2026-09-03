{hostname, ...}: {
  services.openssh.enable = true;

  networking = {
    hostName = hostname;
    interfaces."wlan0".useDHCP = true;

    wireless = {
      enable = true;
      interfaces = ["wlan0"];

      networks = {
        "miwifi".psk = "contrasenha";
      };
    };
  };

  # publish gbeed02.local so the console can be reached without knowing its ip
  services.avahi = {
    enable = true;
    nssmdns4 = true;
    openFirewall = true;

    publish = {
      enable = true;
      addresses = true;
      workstation = true;
    };
  };
}
