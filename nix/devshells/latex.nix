{
  mkShell,
  texliveBasic,
  commonPackages,
}: let
  # basic with pkgs to avoid pulling in the full texlive version, more than 5GB
  latexPackages = texliveBasic.withPackages (ps:
    with ps; [
      latexmk
      xetex
      tools
      colortbl
      graphics-def
      graphics-cfg

      # fonts and languages
      fontspec
      polyglossia
      hyphen-galician
      hyphen-english
      libertine
      psnfss
      helvetic
      zapfding

      # style
      appendix
      caption
      datetime2
      datetime2-galician
      datetime2-english
      fancyhdr
      footnotebackref
      geometry
      lettrine
      listings
      multirow
      setspace
      silence
      titlesec
      tocbibind
      xcolor

      # biblio
      natbib
      ieeetran
      glossaries
      mfirstuc

      # tables and graphics
      pgf
      pgfgantt
      supertabular

      # maths
      amsmath
      amsfonts
      etoolbox
      hyperref
      blindtext
    ]);
in
  mkShell {
    buildInputs = [
      commonPackages
      latexPackages
    ];

    env = {
      LATEXMKOPTS = "-xelatex";
    };
  }
