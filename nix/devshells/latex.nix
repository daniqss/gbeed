{
  mkShell,
  texliveFull,
  commonPackages,
}:
mkShell {
  buildInputs = [texliveFull commonPackages];

  env = {
    LATEXMKOPTS = "-xelatex";
  };
}
