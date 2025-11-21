# anteproxecto
## título galego
Emulador de GameBoy para sistemas embebidos

## título castelán
Emulador de GameBoy para sistemas empotrados

## título inglés
GameBoy Emulator for embedded systems

## Breve descrición
Este proxecto consiste no desenvolvemento dun emulador do modelo DMG de GameBoy que poida ser executado sobre un Linux lixeiro como Alpine Linux nun microcontrolador Raspberry Pi Zero de arquitectura ARM. Para isto debemos replicar o funcionamento dos diferentes compoñentes hardware da consola orixinal: interpretar o conxunto de instrucións da súa CPU, implementar o sistema gráfico, xestionar a entrada/saída... e reproducir a sincronización de todos estes elementos para garantir unha correcta emulación de maneira estable e precisa.
O emulador permitirá xogar con xogos dos tipos de cartuchos máis populares nun contorno lixeiro sen empregar unha sesión gráfica completa, senón usando unha interface sinxela feita cunha libraría que empregue o subsistema de Linux DRM/KMS, como SDL2. Isto proporcionará aos usuarios unha solución optimizada para sistemas limitados e facilitará a súa integración en proxectos embebidos personalizados.

## Obxectivos concretos
- Crear un emulador do modelo DMG de GameBoy completo e funcional
- Comprobar o seu correcto funcionamento cos tipos de cartuchos máis populares
- Integrar o emulador nun sistema embebido ARM sobre un Linux lixeiro

# Método de traballo
Para realizar o proxecto utilizarase a metodoloxía de prototipado rápido, coa que en cada iteración implementarase un compoñente funcional do emulador, validando o seu correcto funcionamento e mellorando o seu deseño segundo se detecten novas necesidades. Deste xeito conseguimos unha fácil integración dos compoñentes para conseguir un emulador completo e funcional. Isto ademais reducirá o esforzo da adaptación final a un contorno embebido. 

# Fases principais
1. Busca e estudo da documentación técnica dispoñible sobre o hardware da GameBoy
2. Análise e deseño da arquitectura do emulador
3. Implementación da CPU cun intérprete do seu conxunto de instrucións
4. Implementación da PPU (Pixel Processing Unit) e os algoritmos de debuxado da pantalla
5. Implementación dos sistemas de entrada/saída e xestión de cartuchos
7. Implementación do sistema de son
6. Integración dos compoñentes e creación dunha interface gráfica sinxela
8. Probas usando diferentes xogos e ROMs de testeo
9. Optimización e adecuación a un contorno embebido
10. Elaboración da documentación e da memoria

# Recursos necesarios
- Unha computadora persoal para o desenvolvemento do software
- Unha Raspberry Pi Zero para as probas en hardware embebido
