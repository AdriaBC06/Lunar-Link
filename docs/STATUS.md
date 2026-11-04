# Estado del proyecto

Última actualización: 2026-11-04.

## Resumen por fase

| Fase | Estado | Contenido |
|---|---|---|
| 1 · Enlace y TUI base | ✅ Hecha | Proyecto Rust, CLI con `clap`, enlace asíncrono con retardo de ida y vuelta, menú con teclado y ratón, `ping`. |
| 2 · Servidor lunar y realismo | ✅ Hecha | Contenido premontado (correo, archivos, galería, operador, telemetría), retardo variable, cortes de señal y ancho de banda limitado. |
| 3 · Animaciones y modo Marte | ✅ Hecha | Panel orbital animado, sparklines, transiciones, escritura progresiva, peticiones concurrentes, `--speed` y paleta de 256 colores. |
| 4 · Portfolio | ✅ Hecha | Licencia, metadatos del crate, CI, workflow de releases, documentación, GIF y capturas. Publicado en crates.io (`lunar-link` 0.1.0), release `v0.1.0` con binarios y página en el portafolio personal. |

## Implementado

**Código** (`src/`)
- `link`: distancia variable (363.000–405.000 km a la Luna, 3–22 minutos luz a Marte); cortes deterministas por semilla (cara oculta cada 90 s, aleatorios y tormentas solares); ancho de banda de 24 KB/s y aceleración del tiempo de viaje.
- `server` y `content`: servidor simulado con contenido fijo en `src/content.toml`. Incluye 5 correos, 5 archivos, 5 ilustraciones ASCII, un árbol de diálogo con palabras clave y telemetría con ciclo día/noche. El traceroute depende del destino.
- `app`: estado de cada sección, hasta 8 peticiones en vuelo emparejadas por id, avisos de respuesta y manejo de la entrada.
- `ui`: panel orbital con estrellas, cuerpos texturizados y paquetes; listas, gráficas, transiciones y aviso de terminal pequeña. El layout está verificado a 64×24.
- `fmt`: números y duraciones en formato español (coma decimal, punto de miles).
- 24 tests: retardo, rango de distancia, cortes, reintentos, descargas, aceleración, contenido, palabras clave, reglas de entrada, formato y renderizado de cada sección al tamaño mínimo. `cargo clippy --all-targets -- -D warnings` pasa sin avisos.

**Proyecto**
- `LICENSE` (GPL-3.0), metadatos en `Cargo.toml` y `README.md` con instalación, uso y controles.
- `.github/workflows/ci.yml` (fmt, clippy, test) y `.github/workflows/release.yml` (binarios para Linux, macOS y Windows al publicar una etiqueta `v*`).
- `docs/ARCHITECTURE.md` con diagramas Mermaid y `docs/PLANNING.md` con el plan por fases.
- `docs/demo.gif` y cuatro capturas generadas con `vhs` a partir de los `.tape` de `docs/`.

## Publicación

- crates.io: `lunar-link` 0.1.0 (`cargo install lunar-link`).
- GitHub: repositorio público con topics y release `v0.1.0`, con binarios para Linux (x86_64 y ARM64), macOS (Intel y Apple Silicon) y Windows, cada uno con su `.sha256`. El metadato `[package.metadata.binstall]` apunta a esos archivos.
- Portafolio: https://adriabc.top/lunar-link/

## Pendiente

- Probar `cargo binstall lunar-link`.
- El README que muestra crates.io para la 0.1.0 es anterior a algunos cambios de redacción; se actualizará en la siguiente versión.

## Sin verificar

- La restauración de la terminal tras un `panic` no se ha forzado en una prueba manual. `ratatui::init` restaura la pantalla y un hook propio libera la captura del ratón.
- Los binarios de Windows, macOS y Linux ARM64 de la release se compilan en CI, pero no se han ejecutado. Solo se probó el de Linux x86_64.

## Limitaciones conocidas

- El **día lunar** (90 s) y la **órbita del relé** (90 s) están comprimidos para que se noten en una sesión; los reales duran días.
- En **modo Marte** el servidor sigue siendo Base Tranquilidad, con su correo y su telemetría lunar. Cambian el retardo, el traceroute y los cortes (conjunción solar). La distancia a Marte varía con un período de 10 minutos reales.
- Mientras un corte retiene una petición, la barra del paquete sigue avanzando según el tiempo estimado; el panel indica que no hay señal.
- `Esc` en el menú principal cierra la aplicación.
- Las descargas son simuladas: no se escribe ningún archivo en disco.

## Cómo probar el proyecto

**Requisitos:** Rust 1.88+ y una terminal de al menos 64×24, preferiblemente con color verdadero.

```
cargo test
cargo clippy --all-targets -- -D warnings
cargo run --release                     # Tierra → Luna
```

**Recorrido manual** (modo Luna):
1. **Ping** → `Enter`: el paquete ◆ viaja a la Luna y vuelve como ◇ en ~2,6 s. Pulsa `t` para el traceroute.
2. **Correo** → `Enter` sobre un mensaje: el texto se escribe progresivamente y se desplaza con `↑` `↓`.
3. **Archivos** → `Enter` sobre un archivo: barra de descarga limitada por el ancho de banda; aparece ✔ al terminar.
4. **Telemetría**: se actualiza sola e indica la antigüedad de cada dato.
5. **Operador**: elige opciones con `↑` `↓` o escribe «hielo», «señal» o «marte» y pulsa `Enter`.
6. **Galería**: cambia de imagen con `←` `→`.
7. Espera ~80 s para ver el corte por **cara oculta**: la barra de estado se pone en rojo, aparece ✕ en el panel y el ping espera y se reintenta.
8. Reduce la terminal por debajo de 64×24 para ver el aviso, y usa el ratón en el menú.

**Variantes**
```
cargo run --release -- --target marte --speed 100   # lanza Correo y Archivos y sigue navegando
cargo run --release -- --delay 5                    # retardo fijo de 5 s por trayecto
cargo run --release -- --no-outages                 # sin cortes
COLORTERM= cargo run --release                      # fuerza la paleta de 256 colores
```

**Regenerar el GIF y las capturas** (necesita `vhs`, `ttyd` y `ffmpeg`):
```
cargo build --release
vhs docs/demo.tape                # docs/demo.gif
vhs docs/screenshots.tape         # docs/screenshot-luna.png
vhs docs/screenshot-marte.tape    # docs/screenshot-marte.png
vhs docs/screenshot-galeria.tape  # docs/screenshot-galeria.png
vhs docs/outage.tape              # docs/screenshot-outage.png (tarda ~90 s)
```
