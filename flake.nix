{
  description = "DMG Game Boy Emulator for embedded devices";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix/monthly";

    # used in rpi02 with gamepi13 image, that this flake builds
    nixpkgs-pi.url = "github:nixos/nixpkgs/nixos-25.11";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    ...
  }: let
    outputs = self;

    eachSystem = f:
      nixpkgs.lib.genAttrs ["x86_64-linux" "aarch64-linux"]
      (system: f system (import nixpkgs {inherit system;}));
  in {
    lib = import ./nix/lib {inherit inputs outputs;};

    overlays = import ./nix/overlays {inherit inputs outputs;};

    packages = eachSystem (system: pkgs: import ./nix/packages {inherit inputs outputs system pkgs;});
    devShells = eachSystem (system: pkgs: import ./nix/devshells {inherit inputs outputs system pkgs;});

    nixosConfigurations = import ./nix/hosts {inherit inputs outputs;};
    installerImages.gbeed02 = outputs.nixosConfigurations.gbeed02.config.system.build.sdImage;
  };
}
