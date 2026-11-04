# 📋 Planning de Lunar Link

Servidor lunar simulado (sin IA, contenido premontado) con TUI bonita, animada e interactiva.

Cada fase debe compilar y ejecutarse, y termina en un punto de verificación.

## Fase 1 · Enlace y TUI base
- Proyecto Rust con licencia GPL-3.0 y módulos `link`, `server`, `content`, `ui`, `app`.
- Argumentos con `clap` (`--target`, `--delay`, `--no-outages`).
- Enlace asíncrono con `tokio`: petición → retardo de ida → servidor → retardo de vuelta.
- TUI con `ratatui` + `crossterm` (teclado y ratón, restauración limpia de la terminal): menú principal, barra de estado e indicador de petición en vuelo con cuenta atrás.
- Servidor con `ping` como primera función.
- **Verificación:** un test mide que el ping tarda ~2,6 s (o 2×`--delay`), y se puede navegar el menú y lanzar un ping en pantalla.

## Fase 2 · Servidor lunar y realismo del enlace
- Contenido en TOML embebido: correo, archivos, diálogos del operador, imágenes ASCII y telemetría con ciclo día/noche lunar.
- Pantallas de Correo, Archivos, Telemetría, Operador, Ping/traceroute y Galería.
- Retardo variable según la distancia (363.000-405.000 km), cortes de señal (aleatorios, cara oculta, tormentas solares) con reintentos, y ancho de banda limitado con descargas que se ralentizan o se pausan.
- **Verificación:** tests de rango del retardo, reintento tras un corte y tiempo de descarga según tamaño; recorrido manual de cada sección.

## Fase 3 · Animaciones y modo Marte
- Campo de estrellas, órbita Tierra-Luna con el paquete viajando, sparklines en vivo, transiciones y efecto de escritura progresiva.
- Degradación elegante en terminales pequeñas o sin color verdadero.
- Modo Marte (3-22 min) con peticiones diferidas que llegan como notificación, y tiempo acelerado (x10, x100) para probarlo.
- **Verificación:** revisión visual en varios tamaños, ~60 FPS sin CPU alta, y en modo Marte acelerado enviar una petición, hacer otra cosa y recibir la respuesta.

## Fase 4 · Portfolio
- **GIF de cabecera** (`docs/demo.gif`, <15 s) generado con `vhs docs/demo.tape`, y capturas de modo Luna, modo Marte y corte de señal.
- Badges, CI en GitHub Actions (`fmt`, `clippy`, `test`), diagrama del flujo en `docs/`.
- Publicación en crates.io, topics del repo (`rust`, `cli`, `tui`, `ratatui`, `latency`, `space`, `simulator`) y release con binarios.
- **Verificación:** `cargo install lunar-link && lunar-link` funciona en una máquina limpia y el README se ve bien en GitHub.

## Ideas opcionales (después)
- Modo servidor real por TCP, para conectar dos terminales a través del enlace simulado.
- Minijuego de aterrizaje, radio lunar, mapa de la superficie.
- Temas de color.
