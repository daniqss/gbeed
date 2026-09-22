# Plantilla de presentación para a defensa do TFG

Proxecto LaTeX (`beamer`) para a defensa do Traballo Fin de Grao. Reproduce o estilo
da memoria (`docs/memoria`): as mesmas cores corporativas, a mesma tipografía, os
logos oficiais na portada e a mesma organización de ficheiros.

A duración prevista da defensa é de **10 minutos, demostración incluída**.
O guión completo, co reparto de tempos diapositiva a diapositiva e o anexo de
posibles preguntas do tribunal, está en `guion.md`.

## Estrutura

  1) Ficheiro de estilo: `estilo_defensa.sty`

  2) Ficheiro principal: `defensa_tfg.tex` (aquí van os datos persoais: nome,
     título, dirección, titulación e mención)

  3) Directorios:

     > `contido/`	Contén as seccións da presentación.
     >
     > `diagramas/`	Diagramas TikZ tomados de `docs/memoria`, sen o entorno
     >               `figure` nin o `\caption`, para poder inserilos nun `frame`.
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
flotantes que colocar nin índice de figuras. Os patróns de uso están
exemplificados no contido:

  - `contido/arquitectura.tex`: figura acompañada de texto en dúas columnas, e
    unha diapositiva `plain` con dúas figuras e nada máis.
  - `contido/implementacion.tex`: imaxe a sangue nun `frame` sen título, e dúas
    figuras apiladas nunha columna.
  - `contido/extra.tex`: diagramas TikZ escalados a unha fracción de
    `\textheight` con `\resizebox`.

Os anchos exprésanse sempre en fraccións de `\textwidth` ou de `\textheight`
(nunca en centímetros), para que a imaxe se adapte ao tamaño da diapositiva e da
columna que a contén.

As imaxes da memoria reutilízanse copiándoas a `imaxes/`. Os diagramas TikZ de
`docs/memoria/diagramas/` están copiados en `diagramas/` sen o entorno `figure`,
o `\caption` e o `\label`, e insírense con `\input` dentro dun `\resizebox`; o pé
ponse coa macro `\fonte`. Como eses diagramas usan acrónimos do paquete
`glossaries`, que aquí non se carga, `defensa_tfg.tex` define `\acrshort` e
compañía para expandilos a maiúsculas.
