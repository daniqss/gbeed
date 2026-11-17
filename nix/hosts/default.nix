{
  inputs,
  outputs,
  ...
}: {
  gbeed02 = inputs.nixos-raspberrypi.lib.nixosSystem {
    specialArgs = {
      inherit inputs outputs;
      hostname = "gbeed02";
      username = "gbeed";
    };
    modules = [
      inputs.nixos-raspberrypi.nixosModules.sd-image
      ./gbeed02/configuration.nix
      ./gbeed02/hardware-configuration.nix
    ];
  };
}
