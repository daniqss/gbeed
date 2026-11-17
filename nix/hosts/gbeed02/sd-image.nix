{
  username,
  config,
  lib,
  ...
}: let
  user = config.users.users.${username};
in {
  # create rom dirs during image creation, so the user can drop roms in before first boot
  sdImage.populateRootCommands = lib.mkAfter ''
    mkdir -p ./files${user.home}/roms ./files${user.home}/saves
  '';
}
