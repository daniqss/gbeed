{outputs, ...}: {
  gbeed02 = outputs.lib.mkGbeedSystem {
    host = ./gbeed02.nix;
    modules = [];
  };
}
