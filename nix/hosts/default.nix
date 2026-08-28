{
  inputs,
  outputs,
  ...
}: {
  gbeed02 = inputs.nixos-raspberrypi.lib.nixosSystemFull {
    # who this host is, shared by every module below instead of being repeated
    # in each of them
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
