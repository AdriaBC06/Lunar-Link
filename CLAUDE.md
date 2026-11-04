# Lunar Link

Pequeño "servidor lunar" simulado, sin IA: contenido premontado al que te conectas desde una TUI con el retardo real de las señales espaciales (Luna ~1,3 s por trayecto, Marte 3-22 min). El objetivo es una TUI bonita, animada e interactiva. Es un proyecto de portfolio.

Ver `README.md` (descripción, uso, arquitectura), `docs/PLANNING.md` (fases y puntos de verificación) y `docs/STATUS.md` (estado actual, pendientes y cómo probar).

## Stack
Rust, `ratatui` + `crossterm`, `tokio`, `clap`, `serde` + TOML. Un único binario.

## Módulos
`link` (retardo, cortes, ancho de banda), `server` (respuestas simuladas), `content` (datos premontados), `ui` (pantallas y animaciones), `app` (estado y eventos).

## Cómo trabajar
- Implementar por fases de `docs/PLANNING.md`, deteniéndose en cada punto de verificación.
- Cada fase debe compilar y ejecutarse; mantener los tests actualizados.
- No añadir IA ni llamadas de red: todo el contenido es fijo y local.
- La terminal debe restaurarse siempre al salir (también ante un panic).
- Priorizar el instalado con un solo comando (`cargo install`).

## Convenciones de git
- Excepción de este proyecto: Claude figura como coautor en los commits. Termina cada mensaje de commit con:
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`
- Rama por defecto: `main`.

## Comandos
```
cargo build
cargo run -- --help
cargo test
cargo clippy
cargo fmt
```
