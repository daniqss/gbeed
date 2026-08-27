{
  pkgs,
  commonPackages,
  platformPackages,
  platformFeatures,
}: let
  latexPackages = pkgs.texliveBasic.withPackages (ps:
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
  pkgs.mkShell {
    packages = commonPackages;
    buildInputs = platformPackages ++ [latexPackages];

    env = {
      DISPLAY_FEATURES = pkgs.lib.concatStringsSep " " platformFeatures;
      RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath platformPackages;

      __GLX_VENDOR_LIBRARY_NAME = "mesa";

      LATEXMKOPTS = "-xelatex";
    };
  }
