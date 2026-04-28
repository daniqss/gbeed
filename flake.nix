{
  description = "DMG Game Boy Emulator for embedded devices";

  nixConfig = {
    extra-substituters = ["https://nixos-raspberrypi.cachix.org"];
    extra-trusted-public-keys = ["nixos-raspberrypi.cachix.org-1:4iMO9LXa8BqhU+Rpg6LQKiGa2lsNh/j2oiYLNOQ5sPI="];
    connect-timeout = 5;
  };

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    fenix.url = "github:nix-community/fenix";
    nixos-raspberrypi.url = "github:nvmd/nixos-raspberrypi/main";
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

    packages = eachSystem (system: pkgs: import ./nix/packages {inherit inputs outputs system pkgs;});
    devShells = eachSystem (system: pkgs: import ./nix/devshells {inherit inputs outputs system pkgs;});

    nixosConfigurations = import ./nix/hosts {inherit inputs outputs;};
    installerImages.gbeed02 = outputs.nixosConfigurations.gbeed02.config.system.build.sdImage;
  };
}
