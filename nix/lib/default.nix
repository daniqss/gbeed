{
  inputs,
  outputs,
}: {
  # build a NixOS configuration for SD Images system for a Raspberry Pi Zero 2 host
  mkSystem = {
    host,
    modules ? [],
    ...
  }:
    inputs.nixos-raspberrypi.lib.nixosSystem {
      specialArgs = inputs // {inherit outputs;};
      modules =
        [
          inputs.nixos-raspberrypi.nixosModules.sd-image
          host
        ]
        ++ modules;
    };
}
