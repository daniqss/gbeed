# taken from https://github.com/plmercereau/nixos-pi-zero-2/blob/main/sd-image.nix, thank you!
# extends the stock sd-image module with `sdImage.extraFirmwareConfig`, since config.txt
# cannot be edited after the image module writes it.
{
  config,
  lib,
  ...
}: {
  options.sdImage.extraFirmwareConfig = lib.mkOption {
    type = lib.types.attrs;
    default = {};
    description = "Extra configuration to be added to config.txt.";
  };

  config = {
    sdImage.extraFirmwareConfig = {
      # nothing is drawn through the vpu, give the ram back to the system
      start_x = 0;
      gpu_mem = 16;
    };

    sdImage.populateFirmwareCommands = lib.mkIf (config.sdImage.extraFirmwareConfig != {}) (
      let
        keyValues =
          lib.mapAttrsToList (name: value: "${name}=${toString value}")
          config.sdImage.extraFirmwareConfig;
      in
        lib.mkAfter ''
          config=firmware/config.txt
          # the file was just created read only, make it appendable
          chmod u+w $config
          printf '\n# extra configuration\n%s\n' ${lib.escapeShellArg (lib.concatStringsSep "\n" keyValues)} >> $config
          chmod u-w $config
        ''
    );
  };
}
