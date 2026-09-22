# Guión defensa — *Emulador de Game Boy para sistemas embebidos*

---

## Diapositiva 1 — Portada, m m mApertura (+-0:30)

Bos días, son Daniel Queijo Seoane. Co permiso do tribunal, procederei coa defensa do meu Traballo Fin de Grao, titulado "Emulador de Game Boy para sistemas embebidos".

Como resumo, o proxecto consiste nun **emulador da Game Boy orixinal**, deseñado como unha
librería reutilizable e portable que puidese ser executada en 
diferentes plataformas, facendo un especial esforzo en sistemas
embebidos de recursos limitados. Sobre el documéntase ademais a construción dunha
consola DIY de referencia.

---

## Diapositiva 2 — Índice (+-0:35)

Esta exposición organízase en sete bloques.

Primeiro comezarei cunha introdución, onde se expoñerá o posicionamento do proxecto e os seus obxectivos.

Despois comentarei a metodoloxía seguida.

A continuación falarei brevemente das tecnoloxías empregadas para o desenvolvemento, para pasar despois a explicar o propio desenvolvemento.

Falarei dalguns dos compoñentes implementados e doutros aspectos relevantes para acadar os obxectibos propostos.
Continuarei falando da validación realizada para asegurar a correcta emulación do hardware.
Concluirei a presentación falando das leccions aprendidas e das liñas futuras de traballo.
Despois, rematarei a defensa cunha demostración do traballo desenvolto.

---

## Diapositiva 4 — Xogar á Game Boy hoxe (+-0:35)

A Game Boy segue sendo unha plataforma con público. Ten unha comunidade de homebrew activa, q desenvolve ferramentas e xogos novos, e existen usuarios que queren seguir disfrutando do catálogo clásico.


O problema principal do hardware orixinal é escaso. a dia de hoxe funciona mais como un articulo de coleccionismo que como unha plataforma de xogo. ademais disto,
require dun investimento en mantemento e reparacións para conservar a funcionalidade
orixinal, e para suplir as demandas de ergonomía modernas, como unha pantalla retroiluminada ou unha batería recargable.

Tamén existen consolas comerciais baseadas en sistemas operativos de proposito xeral como Linux e android que ofrecen unha experiencia de consola portatil moderna. 

O problema de consolas como as de Anbernic e que 
estan sobredimensionadas para o proposito de emular a game boy, sendo moito 
mais potentes do necesario para emular esta consola

Tamen existen proxectos DIY que consiguen unha experiencia de consola portatil con 
hardware mais modesto.
O seu principal problema é a accesibilidade para o usuario medio non tecnico.
estos proxectos adoitan ser recopilacions de diferentes proxectos, pobremente documentados ou empregan software bare metal que dificulta a amplicacion por parte do usuario.
ademais, normalmente requiren soldadura dos diferentes compoñentes empregados



---

## Diapositiva 5 — Obxectivos (+-0:30)


Para cubrir este nicho este traballo propón unha solución implementada dende cero
dun emulador de game boy portable e eficiente para ser empregado en todo tipo de contornas. ademais unha construcion de referencia baseada en hardware accesible sobre o que se emprega un sistema operativo Linux,
mantendo unha balanza entre a extensibildade para os usuarios avanzados e a accesibilidade para os usuarios medios.

Esta levar a cabo esta proposta definiuse nos seguintes obxectivos concretos:


- Desenvolver un emulador do modelo orixinal de Game Boy, o DMG-01 (Dot Matrix Game).
- Validar os seus compoñentes fundamentais contra un conxunto de probas de referencia amplamente usado pola comunidade.
- Garantir o seu correcto funcionamento nos xogos máis populares da consola, usando
unha mostra representativa de cartuchos con diferentes características de hardware.
- Deseñar e desenvolver unha interface axeitada para o seu uso en contornas embebidas.
- Optimizar o proxecto para asegurar a súa correcta execución en contornas de recursos limitados.
- Documentar unha construción de referencia sobre hardware accesible, que permita a calquera persoa montar, adaptar ou mellorar o seu propio dispositivo
---

## Diapositiva 7 — Desenvolvemento iterativo e incremental (+-0:30)

Antes de entrar no sistema, un apunte sobre como se construíu. Seguín un
desenvolvemento **iterativo e incremental**, en once iteracións, unha por
compoñente da consola.

A elección non é casual: a orde vén imposta polo **acoplamento do propio
hardware**. Sen unha CPU que execute código correctamente non se pode desenvolver
a unidade gráfica, e sen unidade gráfica non hai forma de validala. Cada iteración
seguiu sempre a mesma estrutura —estudo da documentación, análise da arquitectura,
implementación e validación— e non se pasaba á seguinte ata que a anterior
estaba consolidada, para evitar que os erros se propagasen.

No cronograma vense as once iteracións entre outubro e xuño, coa documentación en
paralelo á parte final. Para o control de versións usei Git e GitHub, mantendo
sempre a rama principal estable.

---

## Diapositiva 9 — A consola que hai que emular (+-0:25)

Entramos no desenvolvemento. Primeiro, que hai que emular.

Case todo o sistema vive nun **único SoC**: a CPU Sharp SM83, a unidade de
procesamento de píxeles, a de audio, os temporizadores, o joypad e o controlador
de interrupcións. Todos eles colgan dun bus de datos de oito bits e un bus de
direccións de dezaseis, o que dá **64 KiB de espazo compartido**.

E ese espazo non é só memoria: **a entrada/saída está mapeada en memoria**, así que
unha mesma escritura pode ir á RAM, á memoria de vídeo ou a un rexistro de
control segundo a dirección. Por último, o cartucho tampouco é só ROM: leva o seu
propio hardware dentro.

---

## Diapositiva 10 *(opcional)* — Placa base e mapa de memoria (+-0:15)

Aquí vense as dúas cousas xuntas: á esquerda a placa base real, co SoC, a RAM e a
VRAM sinaladas; á dereita o mapa de memoria completo, que é o que hai que
reproducir no emulador.

---

## Diapositiva 11 — Un núcleo independente da plataforma (+-0:30)

O deseño do núcleo respondeu a tres directrices.

**Precisión**, pero ponderada fronte á simplicidade: escollín implementacións máis
sinxelas cando a ganancia en precisión só afectaba a un subconxunto pequeno e
pouco representativo do catálogo.

**Portabilidade**: o núcleo compila en `no_std`, é dicir, sen a biblioteca estándar
de Rust e sen ningunha dependencia externa, porque a biblioteca estándar asume a
presenza dun sistema operativo.

E **rendemento**: no bucle quente non hai reservas de memoria nin chamadas ao
sistema, que son especialmente destrutivas nun procesador dun só núcleo.

Iso tradúcese na organización que se ve á dereita: catro *crates* —o núcleo, as
dúas vistas e as utilidades comúns— e, dentro do núcleo, un módulo por compoñente,
composto dentro do `struct` `Dmg`.

---

## Diapositiva 12 — O trait `Controller` (+-0:25)

A peza que fai posible esa independencia é o *trait* `Controller`. Non define
métodos propios: agrupa nun só tipo as tres operacións de entrada/saída da
consola —debuxar, o porto serie e o audio.

Así, **o núcleo non sabe sobre que plataforma se executa, pero si sabe onde ten que
delegar**. Que o cadro se debuxe nunha textura de raylib, nun *framebuffer* en
memoria ou que simplemente se descarte é decisión do cliente. As propias probas
implementan este *trait* para observar a execución dende fóra, sen ensuciar o
núcleo con código que só ten sentido durante a validación.

E esta flexibilidade non custa nada en execución: `run` e `step` reciben un
**parámetro xenérico**, non un obxecto de *trait*, polo que o compilador
monomorfiza o código e non hai chamadas virtuais.

---

## Diapositiva 13 — Da instrución ao seu efecto (+-0:25)

Este é o patrón co que está implementado o conxunto de instrucións. Cada
instrución é un `struct` que implementa o *trait* `Instruction`, e o operando é un
**parámetro xenérico** que xa leva consigo o seu tamaño e o seu custo en ciclos.
Iso elimina practicamente todo o código repetido sen custo en tempo de execución.

O resultado da execución non se aplica directamente: devólvese nun
`InstructionEffect` co tempo consumido e coas bandeiras. E as bandeiras necesitan
un tratamento especial, porque cada unha pode quedar a `true`, a `false` ou
**sen cambios**; por iso son campos opcionais.

---

## Diapositiva 14 *(opcional)* — Un paso do procesador (+-0:20)

Este é o fluxo dun paso completo. O primeiro non é executar: é **atender as
interrupcións** pendentes por orde de prioridade. Só se non as hai se busca, se
decodifica e se executa a instrución.

E o importante do final: o tempo consumido pola instrución é o que despois fai
avanzar a unidade gráfica, a de audio, os temporizadores e o porto serie.

---

## Diapositiva 15 — A unidade gráfica (+-0:25)

A unidade de procesamento de píxeles é unha máquina de estados que vai percorrendo
os seus modos. O algoritmo de debuxado é o **scanline rendering**: cada liña
debúxase enteira, de esquerda a dereita, capa por capa.

Execútanse tres funcións en orde, e só se a capa está activa: fondo, xanela e
obxectos. O fondo garda unha **caché cos identificadores de cor da liña**, que é o
que despois permite resolver as regras de prioridade dos obxectos sobre el.

É certo que esta aproximación é menos fiel có algoritmo do *FIFO fetcher*, que
reproduce os efectos de modificar rexistros a media liña. Escollina pola súa
sinxeleza e porque, na práctica, o catálogo que necesita esa precisión reduciuse a
casos moi coñecidos e illados.

---

## Diapositiva 16 *(opcional)* — VRAM e OAM (+-0:10)

Estas son as texturas de depuración da primeira interface: o contido da memoria de
vídeo e os obxectos da OAM. Foron imprescindibles para distinguir se un erro
estaba na composición do cadro ou nos datos gardados en memoria.

---

## Diapositiva 17 — Optimización (+-0:25)

Para que isto entre nunha Raspberry Pi Zero houbo que optimizar. Partín de
*flamegraphs* xerados co perfilador `perf`, para concentrar o esforzo onde
realmente custaba.

Tres grupos de melloras. Nos **bucles de debuxado**, reutilizar cálculos entre
píxeles consecutivos —a fila de *tiles* é a mesma para os cento sesenta píxeles da
liña— e acceder directamente ás memorias da unidade gráfica. En
**complexidade algorítmica**, decatarme de que o temporizador e a unidade de audio
son contadores periódicos: o número de eventos dun intervalo calcúlase de forma
directa, e o que era lineal no número de ciclos pasa a ser constante. E no
**bucle quente**, eliminar `Box` e o despache dinámico, e quitar os avisos por
consola que se disparaban cos rexistros de Game Boy Color.

Só coa primeira, o debuxado pasou de supoñer o 26 % do tempo de execución ao 9,5 %.

---

## Diapositiva 18 — Dúas interfaces sobre o mesmo núcleo (+-0:25)

Sobre ese núcleo construínse dúas vistas.

A primeira está orientada á **depuración**: amosa texturas co estado da memoria de
vídeo e dos rexistros da unidade gráfica. Como o núcleo non depende do navegador,
esta vista compílase tamén a WebAssembly con Emscripten e despregase soa en GitHub
Pages con cada confirmación na rama principal.

A segunda é a de **estilo consola**, pensada para a pantalla pequena: unha máquina
de estados con catro pantallas, selección de ROMs, paletas de cores e control de
velocidade. Para poder acelerar a execución sen malgastar recursos debuxando
cadros que a pantalla non pode amosar, implementei un acumulador de tempo que
**desacopla a velocidade de execución da de debuxado**.

---

## Diapositiva 19 — Do emulador á consola (+-0:25)

A construción de referencia usa dous modelos de Raspberry Pi Zero, a W e a Zero 2
W, o que permite validar o mesmo núcleo en **dúas arquitecturas distintas**,
`armv6l` e `aarch64`.

A parte de consola ponna o GamePi13: unha placa coa pantalla SPI, o altofalante e
os botóns, que se conecta directamente aos GPIO **sen soldadura**. En total, uns
cincuenta euros de material fronte aos case douscentos corenta dólares dunha
Analogue Pocket.

---

## Diapositiva 20 — Unha imaxe de sistema reproducible (+-0:25)

Para o despregamento hai dúas vías.

A recomendada é unha **imaxe de NixOS** construída dende o propio repositorio. O
soporte da pantalla resólvese co controlador `panel-mipi-dbi` do núcleo de Linux,
no canto das APIs obsoletas que recomenda o fabricante; o *firmware* coa secuencia
de inicialización do controlador ST7789V xérase en tempo de compilación. O
resultado é que **a pantalla aparece como un dispositivo DRM máis**, así que raylib
debuxa directamente nela sen necesidade dunha sesión gráfica completa. O usuario
só grava a imaxe na tarxeta e acende o dispositivo.

A segunda vía é a única posible para a Zero orixinal, porque `armv6l` non está
soportada por NixOS: parte da imaxe de Debian do fabricante. Aquí materializouse
un dos riscos previstos na planificación —`fbcp` só funciona sobre X11, así que
houbo que cambiar o *backend* DRM polo de X11— pero o impacto para o usuario final
foi mínimo.

---

## Diapositiva 22 — Tres niveis de validación (+-0:30)

A validación fíxose en tres niveis.

**Probas unitarias** sobre as instrucións máis complexas da CPU, que é o primeiro
compoñente e a base de todo o demais.

**Suites de referencia da comunidade**, que son probas validadas en hardware real:
as de Blargg para o conxunto de instrucións, os tempos e o son; a Mooneye Test
Suite para os controladores de bancos de memoria; e dmg-acid2 para a unidade
gráfica, que é a imaxe que se ve á dereita e que require revisión visual.

E finalmente os **xogos comerciais**, a proba definitiva.

Un detalle que me parece relevante: as dúas primeiras suites publican o seu
veredicto polo **porto serie**. Ese foi o motivo principal para emulalo, e é o que
permite que a batería enteira se execute soa en GitHub Actions con cada
confirmación.

---

## Diapositiva 23 — Validación con xogos comerciais (+-0:20)

Superar as suites non garante que un xogo real funcione, porque as suites proban
comportamentos illados. Por iso a última fase foi executar xogos comerciais.

A mostra escolleuse entre os **máis vendidos do catálogo**, que son tamén os que
máis xente vai querer executar, e cobre todos os tipos de cartucho implementados.
Todos son xogables. A única excepción é Pokémon Crystal, que **é exclusivo de Game
Boy Color**: tampouco arrinca nunha Game Boy real, así que queda fóra do alcance do
traballo. Si funcionan os retrocompatibles, como Pokémon Gold.

---

## Diapositivas 24–25 — Demostración (+-2:00)

Máis que seguir contándoo, prefiro amosalo. Vou percorrer catro pasos en voz alta.

**Paso 1 — Arrancar.** Acendo o dispositivo. É a imaxe de NixOS gravada na tarxeta,
sen ningunha configuración manual: arranca directamente na interface de estilo
consola, no menú de selección de ROMs.

**Paso 2 — Xogar.** Cargo un xogo e xogo uns segundos. Aquí vese o debuxado a
velocidade real, a entrada polos botóns da placa a través dos GPIO e o son pola
saída PWM do altofalante.

**Paso 3 — Os menús.** Abro o menú do xogo e cambio a paleta de cores. Subo a
velocidade de execución: esta é a parte na que se nota o acumulador que desacopla
execución e debuxado.

**Paso 4 — Persistencia.** Gardo a partida, saio ao menú e recupéroa. Isto é a RAM
externa do cartucho, que o núcleo expón e a vista volca a un ficheiro.

*(Se sobra tempo: amosar tamén a versión web no navegador, coas texturas de
depuración, para ver que é o mesmo núcleo.)*

> **Plan B se a demo falla:** volver á diapositiva 19 e explicar a construción coa
> fotografía, e á 18 coas capturas da versión web. O discurso é o mesmo; o que se
> perde é a execución en vivo, non o contido.

---

## Diapositiva 27 — Obxectivos cumpridos (+-0:35)

Volvo ás diapositivas para recapitular. O traballo acada os obxectivos propostos.

A **emulación está validada**: o núcleo do DMG-01 está completo e comprobado contra
as suites de referencia e contra os xogos máis vendidos do catálogo. O **núcleo é
reutilizable**: é unha librería `no_std`, sen dependencias e independente de
calquera librería gráfica, con dúas vistas de referencia construídas sobre ela.
**Execútase en recursos limitados**, a velocidade real nunha Raspberry Pi Zero. E
sobre el documentouse unha **consola DIY** con hardware accesible, unha imaxe de
sistema reproducible e licenza GPLv2.

A nivel persoal, implementar dende cero conceptos como a entrada/saída mapeada en
memoria ou un conxunto de instrucións completo asenta dun xeito moito máis
duradeiro o que na carreira se ve sobre todo de forma teórica.

---

## Diapositiva 28 — Liñas de traballo futuro (+-0:25)

De cara ao futuro hai varias liñas claras.

A máis obvia, o **soporte de Game Boy Color**, que multiplicaría o catálogo
aproveitando boa parte do traballo xa feito. En **precisión**, pasar a unha
execución ciclo a ciclo na CPU e ofrecer o *FIFO fetcher* como opción. En
**rendemento**, compilar en tempo de execución os bloques de código máis repetidos.
En **portabilidade**, unha interface como librería de C para consumir o núcleo
dende Python, Java ou C#. E na experiencia de usuario, unha carcasa impresa en 3D e
unha web de documentación fóra da forxa de código.

O relevante é que a separación entre o núcleo e as vistas permite abordalas
**sen tocar a arquitectura**.

---

## Diapositiva 29 — Peche (+-0:10)

E ata aquí a miña exposición. Con isto dou por rematada a defensa do meu Traballo
Fin de Grao. Moitas grazas pola vosa atención.

---
---

# Anexo · Posibles preguntas do tribunal

As diapositivas 30 a 39 do PDF son material de apoio para esta rolda: conxunto de
instrucións do SM83, modos da PPU, controladores de bancos de memoria, cabeceira do
cartucho, máquina de estados da interface, entrada polos GPIO, o exemplo de
optimización do temporizador, a arquitectura de raylib, a unidade de audio e os
custos.

**Por que Rust e non C ou C++?**
O rendemento é equivalente, así que a diferenza está noutro sitio. As garantías de
seguridade de memoria evitan toda unha clase de erros difíciles de depurar, e sobre
todo o sistema de tipos e o *pattern matching* exhaustivo permiten modelar o
hardware, o conxunto de instrucións e as máquinas de estados de forma directa, coa
comprobación feita en tempo de compilación. Ademais, `no_std` é exactamente o
mecanismo que necesitaba para un núcleo embebible.

**Que precisión ten o emulador? Por que non ciclo a ciclo?**
A precisión pondérase fronte á simplicidade. A CPU contabiliza os ciclos por
instrución, non ciclo a ciclo, e a PPU usa *scanline rendering* en vez do *FIFO
fetcher*. Son decisións conscientes: cubrir o que necesita a práctica totalidade do
catálogo comercial. A execución ciclo a ciclo e o *FIFO fetcher* opcional son a
primeira liña de traballo futuro en precisión, e son tamén o requisito previo para
o soporte de Game Boy Color.

**Por que non un MMU centralizado para o espazo de direccións?**
Porque acabaría monopolizando a xestión do emulador e rompería a responsabilidade
única: os rexistros deixarían de ser campos dos seus compoñentes. A solución foi
usar o propio contedor `Dmg` como acceso centralizado, cos *traits* `Accessible` e
`Accessible16`. Cada compoñente accede directamente á súa memoria e, á vez, expona
ao resto; `Dmg` só decide, segundo o rango da dirección, a quen lle corresponde
resolver o acceso.

**Por que se emulou o porto serie se case ningún emulador o fai?**
Porque non é para o usuario: é a infraestrutura de validación. As suites de Blargg
e de Mooneye publican por aí os seus resultados, e sen iso non hai forma de
automatizar as probas en integración continua. A implementación só cobre o rol de
mestre, que é o que necesitan esas ROMs; sen cable conectado, o comportamento
coincide co do hardware real.

**A Mooneye tamén sinala os resultados con `LD B, B`. Por que non usalo?**
Porque `LD B, B` non ten ningún efecto en hardware real; é unha convención que
algúns emuladores usan como punto de parada de depuración. Detectalo obrigaría a
meter no núcleo código que só ten sentido durante a validación. Preferín facer a
detección unicamente polo porto serie e manter o núcleo fiel ao comportamento da
consola.

**Como se evita que a flexibilidade do `Controller` custe rendemento?**
`run` e `step` non reciben un obxecto de *trait*, senón un parámetro xenérico
acoutado por `Controller`. O compilador monomorfiza o código para o tipo concreto
de cada cliente, así que a resolución das chamadas ocorre en compilación: non hai
indireccións por táboas de métodos virtuais e o compilador pode optimizar mellor.
O mesmo criterio se aplicou aos MBC, agrupados nun `enum` en vez de nun obxecto de
*trait*, porque ese camiño se percorre en cada lectura de memoria.

**Que aportou realmente o perfilado?**
Evitou optimizar a cegas. O primeiro *flamegraph* amosou que os bucles de debuxado
levaban o 26 % do tempo, e alí é onde ten sentido atacar, porque calquera traballo
redundante se multiplica por máis de vinte e tres mil veces por cadro. Despois de
reutilizar cálculos entre píxeles quedou no 9,5 %. Outro caso revelador foi o dos
xogos retrocompatibles de Game Boy Color: o custo real non estaba na emulación,
senón nun aviso por consola que se imprimía en cada acceso a rexistros que o
emulador non contemplaba.

**Por que dúas imaxes de sistema en vez dunha?**
Porque son dúas arquitecturas. A Raspberry Pi Zero orixinal é `armv6l`, que NixOS
non soporta, así que para ela hai que partir da imaxe de Debian do fabricante. Para
a Zero 2 W, que é `aarch64`, si se pode construír a imaxe completa dende o
repositorio, e esa é a experiencia recomendada: gravar e acender.

**Por que non usar as APIs que documenta o fabricante da pantalla?**
Porque están obsoletas. O método do fabricante depende de `fbcp` e de DispmanX, unha
API propia de Broadcom, e obriga a manter desactivado o controlador moderno de
vídeo e a executar todo sobre X11. Usando `panel-mipi-dbi`, que é xenérico e está
no núcleo de Linux, a pantalla pasa a ser un dispositivo DRM normal e o emulador
debuxa nela sen sesión gráfica, o que é bastante máis eficiente nun dispositivo así.

**Por que raylib e non SDL?**
SDL sería igual de válida e é máis madura. Escollín raylib porque cobre vídeo,
audio e entrada nunha única librería autocontida, cunha API máis sinxela, e porque
a súa organización por capas cobre exactamente as catro contornas que necesitaba:
escritorio, web, X11 e DRM, escollendo o *backend* en tempo de compilación. En todo
caso, esa dependencia queda confinada nas vistas: o núcleo non a coñece.

**Limitacións coñecidas.**
Só se emula o DMG-01, non a Game Boy Color. A PPU non reproduce os efectos de
modificar rexistros a media liña. Quedaron fóra os controladores de cartucho pouco
habituais, como o MBC6, o MBC7 ou a Pocket Camera. E a transferencia serie só
implementa o rol de mestre.

**Relación coa mención en Enxeñaría de Computadores.**
É transversal: arquitectura de computadores e programación de baixo nivel no
núcleo; perfilado e optimización na fase de rendemento; procesamento de sinal no
filtro paso alto da unidade de audio; e sistemas Linux, compilación cruzada e
soporte de hardware na parte embebida.
