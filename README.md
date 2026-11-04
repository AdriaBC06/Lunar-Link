<div align="center">

# 🌙 Lunar Link

![License](https://img.shields.io/badge/license-GPL--3.0-blue)
![Rust](https://img.shields.io/badge/rust-stable-orange?logo=rust)
![TUI](https://img.shields.io/badge/ui-ratatui-8a2be2)
![CI](https://github.com/AdriaBC06/Lunar-Link/actions/workflows/ci.yml/badge.svg)

</div>

> Un pequeño servidor lunar al que te conectas desde la terminal. Sin IA: contenido premontado, un enlace con el retardo real de las señales espaciales y una interfaz de terminal animada e interactiva.

<p align="center"><img src="docs/demo.gif" alt="Demo de Lunar Link" width="800"></p>

*El GIF mostrará la TUI en acción: la petición sale, viaja 1,3 s hasta la Luna, el servidor responde y la respuesta vuelve otros 1,3 s.*

---

## ¿De qué va esto?

La Luna está a unos 384.000 km de la Tierra. La luz tarda unos **1,3 segundos** en llegar, así que cualquier consulta a un servidor alojado allí tendría un viaje de ida y vuelta de unos **2,6 segundos**. A Marte, el retardo va de **3 a 22 minutos solo de ida**.

Con el debate sobre los centros de datos (energía, agua, rechazo vecinal) surge una idea tan loca como divertida: *¿y si los construimos en el espacio?* **Lunar Link** no construye ese centro de datos; te deja **sentir** cómo sería conectarte a un servidor lunar, a través de una terminal.

## ¿Cómo funciona?

Tú → enlace (retardo de ida) → servidor lunar → enlace (retardo de vuelta) → tú.

1. Navegas por la interfaz y lanzas una petición (abrir un correo, descargar un archivo, hacer ping...).
2. El enlace la retiene el tiempo de viaje hasta la Luna.
3. El servidor lunar responde con contenido premontado.
4. El enlace retiene la respuesta el tiempo de vuelta.
5. Te la entrega, con animaciones durante toda la espera.

## Qué hay en el servidor lunar

El servidor simula la estación **Base Tranquilidad** y ofrece contenido fijo, escrito a mano:

- **Correo**: buzón con mensajes de la tripulación.
- **Archivos**: descargas con barra de progreso limitada por el ancho de banda del enlace.
- **Telemetría**: panel en vivo con temperatura, energía solar, nivel de polvo...
- **Operador**: conversación guionizada por menús y palabras clave (sin IA).
- **Ping / traceroute**: mide tu enlace y muestra los saltos hasta la Luna.
- **Galería**: imágenes ASCII de la superficie lunar.

## Características

- **Retardo realista y variable**: la distancia Tierra-Luna oscila entre ~363.000 y ~405.000 km, así que el retardo cambia suavemente.
- **Cortes de señal**: pérdidas de comunicación aleatorias, cara oculta de la Luna, tormentas solares.
- **Ancho de banda limitado**: los archivos grandes tardan más, como en un enlace real.
- **TUI animada e interactiva**: menús navegables con teclado y ratón, paquetes viajando por una órbita animada, campo de estrellas, barras de progreso, gráficas en vivo y cuenta atrás de la señal.
- **Modo Marte**: retardos de minutos; se trabaja con peticiones diferidas, casi como un correo electrónico.
- **Sin dependencias externas**: no necesita red, claves ni API. Funciona al instante.

## Capturas

| Modo Luna | Modo Marte | Corte de señal |
|---|---|---|
| ![Luna](docs/screenshot-luna.png) | ![Marte](docs/screenshot-marte.png) | ![Corte](docs/screenshot-outage.png) |

## Instalación

```
cargo install lunar-link
lunar-link
```

Un solo binario, un solo comando.

## Uso

```
lunar-link                    # retardo Tierra-Luna (por defecto)
lunar-link --target marte     # retardo Tierra-Marte
lunar-link --delay 5          # retardo fijo en segundos (útil para pruebas)
lunar-link --no-outages       # sin cortes de señal
```

## Stack

- **Rust** (un único binario).
- **ratatui + crossterm** para la interfaz de terminal.
- **tokio** para la gestión asíncrona de retardos y reintentos.
- **serde + TOML** para el contenido premontado.

## Arquitectura

- `link`: modelo del enlace (distancia, latencia variable, cortes, ancho de banda).
- `server`: servidor lunar simulado, que responde a las peticiones con contenido fijo.
- `content`: datos premontados (correo, archivos, telemetría, diálogos del operador).
- `ui`: pantallas, widgets y animaciones de la TUI.
- `app`: estado de la aplicación y bucle de eventos.

Planificación de desarrollo en [`docs/PLANNING.md`](docs/PLANNING.md).

## Licencia

GPL-3.0
