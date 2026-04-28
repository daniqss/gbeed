{
  inputs,
  outputs,
  ...
}: {
  gbeed02 = inputs.nixos-raspberrypi.lib.nixosSystem {
    specialArgs = inputs // {inherit outputs;};
    modules = [
      inputs.nixos-raspberrypi.nixosModules.sd-image
      ./gbeed02.nix
    ];
  };
}
