# Plantilla de presentación para a defensa do TFG

Proxecto LaTeX (`beamer`) para a defensa do Traballo Fin de Grao. Reproduce o estilo
da memoria (`docs/memoria`): as mesmas cores corporativas, a mesma tipografía, os
logos oficiais na portada e a mesma organización de ficheiros.

A duración prevista da defensa é de **20 minutos**, o que equivale
aproximadamente a unha diapositiva por minuto. O reparto orientativo por
seccións está anotado en `defensa_tfg.tex`.

## Estrutura

  1) Ficheiro de estilo: `estilo_defensa.sty`

  2) Ficheiro principal: `defensa_tfg.tex` (aquí van os datos persoais: nome,
     título, dirección, titulación e mención)

  3) Directorios:

     > `contido/`	Contén as seccións da presentación.
     >
     > `imaxes/`	Contén as imaxes da presentación (incluídos os logos da portada).
     >
     > `portada/`	Contén a portada.

O ficheiro `listings-rust.sty` é o mesmo que emprega a memoria e define a
linguaxe Rust para o paquete `listings`.

## Xeración da versión PDF

A compilación faise con `latexmk` e `xelatex`, igual que a memoria, dentro do
contorno de desenvolvemento de LaTeX do proxecto:

```sh
nix develop .#latex
just defensa          # xera defensa_tfg.pdf
```

Receitas dispoñibles:

| Receita              | Descrición                                              |
| -------------------- | ------------------------------------------------------- |
| `just defensa build` | Xera `defensa_tfg.pdf` (é a receita por defecto).       |
| `just defensa watch` | Recompila automaticamente ao gardar calquera ficheiro.  |
| `just defensa view`  | Compila e abre o PDF co visor do sistema.               |
| `just defensa clean` | Elimina o PDF e os ficheiros auxiliares.               |

Tamén se pode executar `just` directamente dentro deste directorio.

## Cores e comandos propios

As tres cores son as da memoria e non se deben cambiar:

| Cor        | RGB             | Uso                                                        |
| ---------- | --------------- | ---------------------------------------------------------- |
| `udcpink`  | `177, 0, 114`   | Regras, viñetas, numeración e elementos destacados.        |
| `ficblue`  | `50, 110, 118`  | Títulos, ligazóns e bloques.                               |
| `udcgray`  | `100, 100, 100` | Texto secundario, pés de imaxe e comentarios de código.    |

`estilo_defensa.sty` define, ademais:

  - `\imaxe[ancho]{ficheiro}{fonte}`: insire unha figura centrada cunha liña de
    procedencia en gris debaixo. É a forma recomendada de usar figuras.
  - `\fonte{texto}`: só a liña de procedencia, para figuras compostas.
  - `\destacado{texto}`: resalta un concepto en `udcpink`.

## Figuras

As diapositivas non usan o entorno `figure`: nunha presentación non hai
flotantes que colocar nin índice de figuras. Os tres patróns de uso están
exemplificados no contido:

  - `contido/arquitectura.tex`: figura que ocupa a diapositiva enteira, e figura
    acompañada de texto en dúas columnas.
  - `contido/validacion.tex`: fotografía a sangue, nun `frame` sen título.

Os anchos exprésanse sempre en fraccións de `\textwidth` (nunca en centímetros),
para que a imaxe se adapte ao tamaño da diapositiva e da columna que a contén.
As imaxes da memoria pódense reutilizar copiándoas a `imaxes/`; os diagramas
TikZ de `docs/memoria/diagramas/` pódense inserir con `\input`.
