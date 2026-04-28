{outputs, ...}: {
  gbeed02 = outputs.lib.mkSystem {
    host = ./gbeed02.nix;
    modules = [];
  };
}
