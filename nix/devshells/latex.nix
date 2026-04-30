{
  mkShell,
  texliveFull,
}:
mkShell {
  buildInputs = [texliveFull];

  env = {
    LATEXMKOPTS = "-xelatex";
  };
}
