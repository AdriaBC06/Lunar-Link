# Estado del proyecto

Última actualización: 2026-11-04. Todo el trabajo está commiteado en local; **nada se ha subido a GitHub todavía**.

## Resumen por fase

| Fase | Estado | Qué incluye |
|---|---|---|
| 1 · Enlace y TUI base | ✅ Hecha | Proyecto Rust, CLI con `clap`, enlace asíncrono con retardo de ida y vuelta, menú navegable con teclado y ratón, `ping`. |
| 2 · Servidor lunar y realismo | ✅ Hecha | Contenido premontado (correo, archivos, galería, operador, telemetría), retardo variable, cortes de señal, ancho de banda limitado. |
| 3 · Animaciones y modo Marte | ✅ Hecha | Panel orbital animado, sparklines, transiciones, escritura progresiva, peticiones concurrentes, `--speed`, degradación a 256 colores. |
| 4 · Portfolio | 🟡 Parcial | Hecho: licencia, metadatos del crate, CI, workflow de releases, docs, GIF y capturas. Pendiente: publicar (ver abajo). |

## Hecho

**Código** (`src/`)
- `link`: distancia Tierra-destino variable (363.000–405.000 km a la Luna; 3–22 min luz a Marte), cortes deterministas por semilla (cara oculta cada 90 s, aleatorios, tormentas solares), ancho de banda de 24 KB/s, aceleración del tiempo de viaje.
- `server` y `content`: servidor simulado con contenido fijo en `src/content.toml` (5 correos, 5 archivos, 4 imágenes ASCII, árbol de diálogo del operador con palabras clave, telemetría con ciclo día/noche).
- `app`: estado de cada sección, hasta 8 peticiones en vuelo emparejadas por id, avisos de respuesta, entrada de teclado.
- `ui`: panel orbital con estrellas y paquetes, listas, gráficas, transiciones, aviso de terminal pequeña (mínimo 64×24).
- 18 tests (retardo, rango de distancia, cortes, reintentos, descargas, aceleración, contenido, reglas de entrada). `cargo clippy --all-targets -- -D warnings` limpio.

**Proyecto y portfolio**
- `LICENSE` (GPL-3.0), metadatos en `Cargo.toml`, `README.md` con badges, instalación, uso y controles.
- `.github/workflows/ci.yml` (fmt, clippy, test) y `.github/workflows/release.yml` (binarios Linux, macOS y Windows al subir una etiqueta `v*`).
- `docs/ARCHITECTURE.md` con diagramas Mermaid, `docs/PLANNING.md` con el plan por fases.
- `docs/demo.gif` (11 s) y `docs/screenshot-luna.png`, `screenshot-marte.png`, `screenshot-outage.png`, generados con `vhs` a partir de los `.tape` de `docs/`.

## Pendiente

Estas acciones son públicas o irreversibles y requieren tu confirmación:

1. **Subir a GitHub:** `git push` (el repo es privado; subir publica el contenido en él).
2. **Descripción y topics del repo:** `gh repo edit --description "..." --add-topic rust --add-topic cli --add-topic tui --add-topic ratatui --add-topic latency --add-topic space --add-topic simulator`.
3. **Publicar en crates.io:** `cargo login` y `cargo publish` (el nombre `lunar-link` estaba libre al 2026-11-04; una versión publicada no se puede borrar). `cargo publish --dry-run` ya pasa.
4. **Release `v0.1.0`:** `git tag v0.1.0 && git push --tags` dispara el workflow de binarios.
5. **Badge de crates.io** en el README, una vez publicado, y cambiar la instalación a `cargo install lunar-link`.
6. **Repo público:** si quieres que el README, el GIF y los badges se vean para otros, hay que hacerlo público (`gh repo edit --visibility public --accept-visibility-change-consequences`).

## Sin verificar

- El **clic de ratón** en el menú (con peticiones concurrentes) no se ha probado.
- Que la terminal se **restaure tras un `panic`** no se ha probado (`ratatui::init` instala el hook, pero no se ha forzado un fallo).
- El workflow de **release** no se ha ejecutado nunca; el de **CI** tampoco, porque no se ha hecho push.
- La compilación en **macOS y Windows** no se ha probado.

## Limitaciones conocidas

- El **día lunar** (90 s) y la **órbita del relé** (90 s) están comprimidos para que se noten en una sesión; los reales duran días.
- En **modo Marte** el contenido sigue siendo el de Base Tranquilidad (menciones a la Luna incluidas); solo cambian el retardo, el destino y los cortes (conjunción solar). El período de variación de la distancia a Marte es de 10 minutos reales.
- Mientras un corte de señal retiene una petición, la barra del paquete sigue avanzando según el tiempo estimado; el panel avisa de que no hay señal.
- `Esc` en el menú principal cierra la aplicación.
- Las descargas son simuladas: no se escribe ningún archivo en disco.

## Cómo probar el proyecto

**Requisitos:** Rust 1.88+ y una terminal de al menos 64×24 (mejor con color verdadero).

```
cd ~/Projects/Lunar-Link
cargo test                              # 18 tests, ~1 s
cargo clippy --all-targets -- -D warnings
cargo run --release                     # Tierra → Luna
```

**Recorrido manual sugerido** (Luna, `cargo run --release`):
1. Menú → **Ping** → `Enter` dos veces: el paquete ◆ viaja a la Luna y ◇ vuelve; tarda ~2,6 s. Pulsa `t` para el traceroute.
2. **Correo** → `Enter` (carga la lista) → `Enter` sobre un mensaje: el texto se escribe letra a letra.
3. **Archivos** → `Enter` sobre un archivo: barra de descarga limitada por el ancho de banda; aparece ✔ al terminar.
4. **Telemetría**: se actualiza sola; mira la antigüedad del dato y las gráficas.
5. **Operador**: elige opciones con `↑↓`, o escribe «hielo», «tortilla» o «marte» y pulsa `Enter`.
6. **Galería**: cambia de imagen con `←` `→`.
7. Espera ~80 s para ver el corte por **cara oculta** (barra de estado en rojo, ✕ en el panel, el ping espera y reintenta).
8. Redimensiona la terminal por debajo de 64×24 para ver el aviso, y haz clic en el menú con el ratón.

**Variantes**
```
cargo run --release -- --target marte --speed 100   # Marte con tiempo ×100; lanza Correo y Archivos y sigue navegando
cargo run --release -- --delay 5                    # retardo fijo de 5 s por trayecto
cargo run --release -- --no-outages                 # sin cortes
COLORTERM= cargo run --release                      # fuerza la paleta de 256 colores
```

**Regenerar el GIF y las capturas** (necesita `vhs`, `ttyd` y `ffmpeg`; ya instalados):
```
cargo build --release
vhs docs/demo.tape               # docs/demo.gif
vhs docs/screenshots.tape        # docs/screenshot-luna.png
vhs docs/screenshot-marte.tape   # docs/screenshot-marte.png
vhs docs/outage.tape             # docs/screenshot-outage.png (tarda ~90 s)
```
