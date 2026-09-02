{
  inputs,
  outputs,
  ...
}: {
  gbeed02 = inputs.nixos-raspberrypi.lib.nixosSystemFull {
    specialArgs =
      inputs
      // {
        inherit outputs;
        hostname = "gbeed02";
        username = "gbeed";
        system = "aarch64-linux";
      };
    modules = [
      inputs.nixos-raspberrypi.nixosModules.sd-image
      ./gbeed02/configuration.nix
      ./gbeed02/hardware-configuration.nix
    ];
  };
}
