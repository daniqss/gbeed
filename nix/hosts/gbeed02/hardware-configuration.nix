# strongly based in https://github.com/plmercereau/nixos-pi-zero-2/blob/main/hardware.nix, thank you!
{
  outputs,
  pkgs,
  lib,
  ...
}: {
  nixpkgs = {
    hostPlatform = "aarch64-linux";

    overlays = [
      (_final: super: {
        makeModulesClosure = x: super.makeModulesClosure (x // {allowMissing = true;});
      })
      outputs.overlays.default
    ];
  };

  boot = {
    kernelPackages = pkgs.linuxPackages_rpi02w;

    # needed to load the gamepi13 panel driver, otherwise the panel stays black
    kernelModules = ["panel-mipi-dbi"];

    # vc4 brings its own hdmi codec, the speaker stays on the bcm2835 headphones card
    kernelParams = ["snd_bcm2835.enable_hdmi=0"];

    initrd.availableKernelModules = ["xhci_pci" "usbhid" "usb_storage"];

    loader = {
      grub.enable = false;
      generic-extlinux-compatible.enable = true;
    };

    swraid.enable = lib.mkForce false;
  };

  hardware = {
    graphics.enable = true;

    enableRedistributableFirmware = lib.mkForce false;
    firmware = [
      pkgs.raspberrypiWirelessFirmware
      pkgs.gbeed.gamepi13-panel
    ];

    deviceTree = {
      enable = true;
      filter = "*2837*";
      overlays = [
        {
          name = "gamepi13-panel";
          dtsFile = ./dts/panel.dts;
        }
        {
          name = "gamepi13-audremap18";
          dtsFile = ./dts/audremap18.dts;
        }
        {
          name = "vc4-kms-v3d";
          dtsFile = ./dts/vc4-kms-v3d.dts;
        }
      ];
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
