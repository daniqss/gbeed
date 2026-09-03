{
  inputs,
  outputs,
  ...
}: {
  gbeed02 = inputs.nixpkgs-pi.lib.nixosSystem {
    specialArgs = {
      inherit inputs outputs;
      hostname = "gbeed02";
      username = "gbeed";
    };
    modules = [
      "${inputs.nixpkgs-pi}/nixos/modules/installer/sd-card/sd-image-aarch64.nix"
      ./gbeed02/configuration.nix
      ./gbeed02/hardware-configuration.nix
    ];
  };
}
