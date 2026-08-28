# The GamePi13 itself: the Zero 2 W board, the SPI panel, the PWM speaker and
# the SD card layout.
{
  outputs,
  nixos-raspberrypi,
  system,
  config,
  pkgs,
  lib,
  ...
}: let
  panelFirmware = outputs.packages.${system}.gamepi13-panel;
  audremap18 = outputs.packages.${system}.gamepi13-audremap18;

  # GamePi13 wiring, BCM pin numbers, read off Waveshare's `waveshare13.dtbo`.
  # The backlight is not switchable: their overlay carries a `rpi_backlight`
  # node with every property commented out, so there is no backlight GPIO to
  # hand to the driver.
  panel = {
    bus = "spi0-0";
    speed = 96000000;
    width = 240;
    height = 240;
    widthMm = 23;
    heightMm = 23;
    resetGpio = 27;
    dcGpio = 25;
  };

  panelDevice = "gbeed-panel";

  flag = {
    enable = true;
  };
  param = value: {
    enable = true;
    inherit value;
  };
in {
  imports = with nixos-raspberrypi.nixosModules; [
    raspberry-pi-02.base
    raspberry-pi-02.display-vc4
  ];

  hardware.graphics.enable = true;

  # tryng to make the image slimmer
  hardware.enableRedistributableFirmware = lib.mkForce false;

  hardware.firmware = [
    panelFirmware
    pkgs.raspberrypiWirelessFirmware
  ];

  hardware.raspberry-pi.config.all = {
    base-dt-params.spi = param "on";

    dt-overlays = {
      mipi-dbi-spi = {
        enable = true;
        params = {
          "${panel.bus}" = flag;
          speed = param panel.speed;

          width = param panel.width;
          height = param panel.height;
          width-mm = param panel.widthMm;
          height-mm = param panel.heightMm;

          reset-gpio = param panel.resetGpio;
          dc-gpio = param panel.dcGpio;

          write-only = flag;
          cpha = flag;
          cpol = flag;
        };
      };

      gbeed-audremap18.enable = true;
    };
  };

  # join our overlay to the stock firmware
  boot.loader.raspberry-pi.firmwarePackage = pkgs.symlinkJoin {
    name = "raspberrypifw-gbeed";
    paths = [
      nixos-raspberrypi.packages.${system}.raspberrypifw
      (pkgs.runCommand "gbeed-overlays" {} ''
        install -Dm444 ${audremap18} \
          $out/share/raspberrypi/boot/overlays/gbeed-audremap18.dtbo
      '')
    ];
  };

  services.udev.extraRules = ''
    SUBSYSTEM=="drm", KERNEL=="card[0-9]*", SUBSYSTEMS=="spi", SYMLINK+="dri/${panelDevice}"
  '';

  environment.etc."asound.conf".text = ''
    pcm.!default {
        type plug
        slave.pcm "hw:Headphones,0"
    }
    ctl.!default {
        type hw
        card Headphones
    }
  '';

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
