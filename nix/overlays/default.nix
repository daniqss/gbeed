{
  inputs,
  outputs,
  ...
}: {
  # the sd image builds gbeed with the nixpkgs of the host itself, so it shares its
  # mesa and glibc instead of dragging a second nixpkgs into the image
  default = final: _prev: {
    gbeed = import ../packages {
      inherit inputs outputs;
      pkgs = final;
    };
  };
}
