{
  inputs,
  outputs,
  pkgs,
  lib,
  ...
}: let
  # gamepi13 wiring, BCM pin numbers, read off waveshare's `waveshare13.dtbo`.
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

  flag = {enable = true;};
  param = value: {
    enable = true;
    inherit value;
  };

  # waveshare's `audremap18.dtbo` is not shipped by the vendor firmware
  audremap18 = pkgs.runCommand "gamepi13-audremap18" {nativeBuildInputs = [pkgs.dtc];} ''
    mkdir -p $out/share/raspberrypi/boot/overlays
    dtc -@ -I dts -O dtb -o $out/share/raspberrypi/boot/overlays/gamepi13-audremap18.dtbo ${./dts/audremap18.dts}
  '';
in {
  imports = with inputs.nixos-raspberrypi.nixosModules; [
    raspberry-pi-02.base
  ];

  nixpkgs.overlays = [outputs.overlays.default];

  boot = {
    # needed to load the gamepi13 panel driver, otherwise the panel stays black
    kernelModules = ["panel-mipi-dbi"];

    # vc4 brings its own hdmi codec, the speaker stays on the bcm2835 headphones card
    kernelParams = ["snd_bcm2835.enable_hdmi=0"];

    swraid.enable = lib.mkForce false;

    supportedFilesystems.zfs = lib.mkForce false;

    loader.raspberry-pi.firmwarePackage = pkgs.symlinkJoin {
      name = "raspberrypifw-gamepi13";
      paths = [
        inputs.nixos-raspberrypi.packages.${pkgs.stdenv.hostPlatform.system}.raspberrypifw
        audremap18
      ];
    };
  };

  hardware = {
    graphics.enable = true;

    enableRedistributableFirmware = lib.mkForce false;
    firmware = [
      pkgs.raspberrypiWirelessFirmware
      pkgs.gbeed.gamepi13-panel
    ];

    raspberry-pi.config.all = {
      options = {
        # nothing is drawn through the vpu, give the ram back to the system
        start_x = param 0;
        gpu_mem = param 16;
      };

      base-dt-params.spi = param "on";

      dt-overlays = {
        # the driver loads `panel.bin` from the overlay's first compatible string
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

        gamepi13-audremap18.enable = true;

        # the default 256MiB cma hangs the zero 2 w on boot with recent kernels.
        # fixed upstream in develop (nvmd/nixos-raspberrypi#254), not yet in main
        vc4-kms-v3d.params.cma-128 = flag;
      };
    };
  };

  zramSwap = {
    enable = true;
    algorithm = "zstd";
  };

  users.groups.gpio = {};

  services.udev.extraRules = ''
    SUBSYSTEM=="drm", KERNEL=="card[0-9]*", SUBSYSTEMS=="spi", SYMLINK+="dri/gbeed-panel"
    KERNEL=="gpiochip*", GROUP="gpio", MODE="0660"
    SUBSYSTEM=="gpiomem", GROUP="gpio", MODE="0660"
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

  system.nixos.tags = ["gbeed" "gamepi13"];
}
