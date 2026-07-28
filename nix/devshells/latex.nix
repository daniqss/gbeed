{
  mkShell,
  texliveBasic,
  commonPackages,
}: let
  # Conxunto mínimo de TeX Live para compilar docs/memoria: só os paquetes
  # que a memoria carrega realmente (véxase memoria_tfg.fls). Evítase
  # texliveFull (~5,8 GiB de closure) para que os despregues sexan rápidos
  # e reproducibles.
  texlive = texliveBasic.withPackages (ps:
    with ps; [
      # motor e automatización da compilación
      latexmk
      xetex
      tools # longtable, multicol, array
      colortbl
      graphics-def
      graphics-cfg
      # tipografía e idiomas (xelatex + galego/inglés)
      fontspec
      polyglossia
      hyphen-galician # sen isto o galego queda sen guionado
      hyphen-english
      libertine
      psnfss
      helvetic # phv: portada
      zapfding
      # estilo do documento (estilo_tfg.sty)
      appendix
      caption # subcaption
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
      # bibliografía (natbib + estilo IEEEtranN) e glosarios
      natbib
      ieeetran
      glossaries
      mfirstuc
      # táboas, gráficos e diagramas de Gantt
      pgf
      pgfgantt
      supertabular
      # matemáticas e utilidades varias
      amsmath
      amsfonts
      etoolbox
      hyperref
      blindtext
    ]);
in
  mkShell {
    buildInputs = [texlive commonPackages];

    env = {
      LATEXMKOPTS = "-xelatex";
    };
  }
