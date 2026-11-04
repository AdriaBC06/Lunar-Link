# 📋 Planning de Lunar Link

Servidor lunar simulado (sin IA, contenido premontado) con TUI bonita, animada e interactiva.

Cada fase debe compilar y ejecutarse, y termina en un punto de verificación.

## Fase 1 · Esqueleto y enlace básico
- Proyecto Rust con licencia GPL-3.0 y módulos `link`, `server`, `content`, `ui`, `app`.
- Parseo de argumentos con `clap` (`--target`, `--delay`, `--no-outages`).
- Canal asíncrono con `tokio`: una petición sale, espera el retardo de ida, el servidor responde y espera el de vuelta.
- Servidor con una sola respuesta fija (`ping`).
- **Verificación:** un test mide que el ping tarda ~2,6 s con el retardo por defecto y 2×`--delay` con retardo fijo.

## Fase 2 · TUI base e interactiva
- Bucle de eventos con `ratatui` + `crossterm` (teclado y ratón, redimensionado, restauración limpia de la terminal).
- Menú principal navegable: Correo, Archivos, Telemetría, Operador, Ping, Galería.
- Barra de estado con distancia, latencia y calidad de señal.
- Indicador de "petición en vuelo" con cuenta atrás.
- **Verificación:** navegar por todo el menú, lanzar un ping y ver el resultado en pantalla.

## Fase 3 · Contenido del servidor lunar
- Contenido en TOML embebido en el binario: correos, archivos, diálogos del operador, imágenes ASCII.
- Pantalla de Correo (lista y lectura), Operador (menús y palabras clave), Galería.
- Telemetría generada con funciones deterministas (ciclo día/noche lunar).
- **Verificación:** tests que cargan todo el contenido, y recorrido manual de cada sección.

## Fase 4 · Realismo del enlace
- Retardo variable según la distancia Tierra-Luna (363.000-405.000 km).
- Cortes de señal: aleatorios, cara oculta y tormentas solares, con reintentos automáticos.
- Ancho de banda limitado: descargas con barra de progreso que se ralentiza o se pausa con los cortes.
- **Verificación:** tests de rango del retardo, de reintento tras un corte y de tiempo de descarga según tamaño.

## Fase 5 · Animaciones y pulido visual
- Campo de estrellas con parpadeo de fondo.
- Visualización orbital Tierra-Luna con el paquete viajando de ida y vuelta.
- Gráficas en vivo (sparklines) de latencia y telemetría.
- Transiciones entre pantallas, efecto de escritura progresiva y paleta de colores coherente.
- Degradación elegante en terminales pequeñas o sin color verdadero.
- **Verificación:** revisión visual en varios tamaños de terminal y 60 FPS estables sin consumo alto de CPU.

## Fase 6 · Modo Marte
- Retardos de 3 a 22 minutos con peticiones diferidas: se encolan, puedes seguir navegando y la respuesta llega después como notificación.
- Opción de acelerar el tiempo (x10, x100) para poder probarlo.
- **Verificación:** con tiempo acelerado, enviar una petición, hacer otra cosa y recibir la respuesta.

## Fase 7 · Pulido para portfolio
- **GIF de cabecera** (`docs/demo.gif`, <15 s, ~800 px de ancho) generado con `vhs` a partir de `docs/demo.tape`: menú → ping → paquete viajando a la Luna y volviendo → descarga con barra de progreso. Regenerable con un solo comando (`vhs docs/demo.tape`).
- **Capturas** en `docs/`: modo Luna, modo Marte y corte de señal (`screenshot-luna.png`, `screenshot-marte.png`, `screenshot-outage.png`).
- Badges (licencia, Rust, ratatui, CI) y workflow de CI en GitHub Actions (`fmt`, `clippy`, `test`).
- README que abre con la historia (centros de datos en el espacio) antes de lo técnico.
- Diagrama del flujo en `docs/`.
- Publicación en crates.io, descripción corta y topics del repo: `rust`, `cli`, `tui`, `ratatui`, `latency`, `space`, `simulator`.
- Release en GitHub con binarios precompilados (Linux, macOS, Windows).
- **Verificación:** `cargo install lunar-link && lunar-link` funciona en una máquina limpia, y el README se ve bien en GitHub con el GIF cargando.

## Ideas opcionales (después)
- Modo servidor real por TCP, para conectar dos terminales a través del enlace simulado.
- Más contenido: minijuego de aterrizaje, radio lunar, mapa de la superficie.
- Temas de color.
