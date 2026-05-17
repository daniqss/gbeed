{pkgs, ...}:
pkgs.writers.writePython3Bin "bench-compare" {
  libraries = with pkgs.python3Packages; [matplotlib];
  flakeIgnore = ["E501" "E402" "W503"];
} (builtins.readFile ./bench-compare.py)
