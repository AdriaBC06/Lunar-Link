//! Renderizado de la TUI.

use std::f64::consts::PI;
use std::time::Duration;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Gauge, List, ListItem, ListState, Paragraph, Sparkline, Wrap,
};
use tokio::time::Instant;

use crate::app::{App, Focus, MenuItem};
use crate::fmt;
use crate::link::Target;

const ACCENT: Color = Color::Rgb(180, 190, 254);
const DIM: Color = Color::Rgb(108, 112, 134);
const OK: Color = Color::Rgb(166, 227, 161);
const WARN: Color = Color::Rgb(243, 139, 168);
const GOLD: Color = Color::Rgb(249, 226, 175);
const TRACK: Color = Color::Rgb(49, 50, 68);
const STAR: Color = Color::Rgb(205, 214, 244);
const EARTH: Color = Color::Rgb(137, 180, 250);
const MOON: Color = Color::Rgb(186, 194, 222);
const MARS: Color = Color::Rgb(250, 179, 135);

pub const MENU_WIDTH: u16 = 24;
pub const MIN_WIDTH: u16 = 64;
pub const MIN_HEIGHT: u16 = 24;

/// Duración de la transición al cambiar de sección.
const WIPE_SECS: f64 = 0.18;
/// Caracteres por segundo del efecto de escritura progresiva.
const TYPE_CPS: f64 = 90.0;

fn areas(area: Rect) -> [Rect; 3] {
    Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .areas(area)
}

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let msg = format!(
            "Terminal demasiado pequeña ({}×{}).\nMínimo {MIN_WIDTH}×{MIN_HEIGHT}.",
            area.width, area.height
        );
        frame.render_widget(
            Paragraph::new(msg)
                .alignment(Alignment::Center)
                .style(Style::new().fg(GOLD)),
            area,
        );
        return;
    }

    let [header, body, footer] = areas(area);
    let [menu, main] =
        Layout::horizontal([Constraint::Length(MENU_WIDTH), Constraint::Min(20)]).areas(body);
    let [orbit, section, log] = Layout::vertical([
        Constraint::Length(8),
        Constraint::Min(6),
        Constraint::Length(5),
    ])
    .areas(main);

    render_status(frame, app, header);
    render_menu(frame, app, menu);
    render_orbit(frame, app, orbit);
    render_section(frame, app, section);
    render_log(frame, app, log);

    let hint = match app.focus {
        Focus::Menu => " ↑↓ navegar · Enter abrir · clic con el ratón · q salir",
        Focus::Content => match app.selected_item() {
            MenuItem::Correo => " ↑↓ elegir · Enter leer · Esc volver",
            MenuItem::Archivos => " ↑↓ elegir · Enter descargar · Esc volver",
            MenuItem::Telemetria => " Esc volver",
            MenuItem::Operador => " escribe y Enter · ↑↓ opciones · Esc volver",
            MenuItem::Ping => " Enter ping · t traceroute · Esc volver",
            MenuItem::Galeria => " ←→ cambiar imagen · Esc volver",
        },
    };
    frame.render_widget(Paragraph::new(hint).style(Style::new().fg(DIM)), footer);

    if !app.truecolor {
        downgrade(frame.buffer_mut());
    }
}

/// Área interior de la lista del menú (para traducir clics en opciones).
pub fn menu_inner(frame_area: Rect) -> Rect {
    let [_, body, _] = areas(frame_area);
    let [menu, _] =
        Layout::horizontal([Constraint::Length(MENU_WIDTH), Constraint::Min(20)]).areas(body);
    block("Menú", false).inner(menu)
}

fn block(title: &str, active: bool) -> Block<'_> {
    let border = if active { ACCENT } else { DIM };
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(border))
        .title(Span::styled(
            format!(" {title} "),
            Style::new().fg(ACCENT).bold(),
        ))
}

/// Convierte los colores RGB a la paleta de 256 para terminales sin color verdadero.
fn downgrade(buf: &mut Buffer) {
    fn to_256(c: Color) -> Color {
        match c {
            Color::Rgb(r, g, b) => {
                let q = |v: u8| (u16::from(v) * 5 + 127) / 255;
                Color::Indexed((16 + 36 * q(r) + 6 * q(g) + q(b)) as u8)
            }
            other => other,
        }
    }
    for cell in &mut buf.content {
        cell.fg = to_256(cell.fg);
        cell.bg = to_256(cell.bg);
    }
}

fn bar(ratio: f64, width: usize) -> String {
    let filled = (ratio.clamp(0.0, 1.0) * width as f64).round() as usize;
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

/// Hash barato y estable para colocar las estrellas.
fn hash(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 12)
}

/// Tramo del texto visible con el efecto de escritura progresiva.
fn typed(text: &str, since: Instant) -> &str {
    let n = (since.elapsed().as_secs_f64() * TYPE_CPS) as usize;
    match text.char_indices().nth(n) {
        Some((i, _)) => &text[..i],
        None => text,
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let t = app.link.elapsed();
    let mut spans = vec![
        Span::styled("🌙 LUNAR LINK", Style::new().fg(ACCENT).bold()),
        Span::styled("  │  ", Style::new().fg(DIM)),
    ];
    match app.link.status_at(t) {
        Some(outage) => {
            let left = outage.ends_at.saturating_sub(t).as_secs_f64().ceil();
            spans.push(Span::styled(
                format!(
                    "⚠ SIN SEÑAL · {} · reconexión en {left:.0} s",
                    outage.kind.label(app.link.target)
                ),
                Style::new().fg(WARN).bold(),
            ));
        }
        None => {
            let q = app.link.signal_quality(t);
            spans.extend([
                Span::styled("● ", Style::new().fg(OK)),
                Span::raw(app.link.target.label()),
                Span::styled("  │  ", Style::new().fg(DIM)),
                Span::raw(format!("{} km", fmt::km(app.link.distance_km(t)))),
                Span::styled("  │  ", Style::new().fg(DIM)),
                Span::raw(format!("ida y vuelta {}", fmt::dur(app.link.round_trip()))),
                Span::styled("  │  ", Style::new().fg(DIM)),
                Span::raw("señal "),
                Span::styled(bar(f64::from(q) / 100.0, 5), Style::new().fg(OK)),
                Span::raw(format!(" {q}%")),
            ]);
        }
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).block(block("Estado", false)),
        area,
    );
}

fn render_menu(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = MenuItem::ALL
        .iter()
        .map(|i| ListItem::new(format!(" {}", i.label())))
        .collect();
    let active = app.focus == Focus::Menu;
    let highlight = if active {
        Style::new()
            .fg(Color::Black)
            .bg(ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
    };
    let list = List::new(items)
        .block(block("Menú", active))
        .highlight_symbol("▶")
        .highlight_style(highlight);
    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

/// Dibuja una cadena recortada al área, sin salirse del búfer.
fn put(buf: &mut Buffer, area: Rect, x: i32, y: i32, text: &str, style: Style) {
    if y < i32::from(area.y) || y >= i32::from(area.y + area.height) {
        return;
    }
    for (i, ch) in text.chars().enumerate() {
        let cx = x + i as i32;
        if cx >= i32::from(area.x) && cx < i32::from(area.x + area.width) {
            buf.set_string(cx as u16, y as u16, ch.to_string(), style);
        }
    }
}

/// Panel animado del enlace: estrellas, Tierra, destino y los paquetes en vuelo.
fn render_orbit(frame: &mut Frame, app: &App, area: Rect) {
    let speed = app.link.speed;
    let title = if speed > 1.0 {
        format!("Enlace ×{speed}")
    } else {
        "Enlace".to_string()
    };
    let b = block(&title, false);
    let inner = b.inner(area);
    frame.render_widget(b, area);
    if inner.width < 24 || inner.height < 5 {
        return;
    }

    let now = app.link.elapsed();
    let t = now.as_secs_f64();
    let outage = app.link.status_at(now);
    let sky_h = i32::from(inner.height) - 2;
    let (x0, y0) = (i32::from(inner.x), i32::from(inner.y));
    let center = y0 + sky_h / 2;
    let w = i32::from(inner.width);

    let (dest_color, dest_label) = match app.link.target {
        Target::Luna => (MOON, "Luna"),
        Target::Marte => (MARS, "Marte"),
    };
    let earth_x = x0 + 1;
    let dest_x = x0 + w - 5;
    let path_start = earth_x + 6;
    let path_end = dest_x - 2;
    let path_len = f64::from(path_end - path_start).max(1.0);

    let buf = frame.buffer_mut();

    // Campo de estrellas parpadeantes.
    for y in 0..sky_h {
        for x in 0..w {
            let h = hash(x as u32, y as u32);
            if !h.is_multiple_of(17) {
                continue;
            }
            let glyph = ["·", ".", "+", "*"][(h >> 5) as usize % 4];
            let phase = f64::from((h >> 8) & 255) * 0.07;
            let bright = (t * 1.4 + phase).sin() > 0.2;
            let color = if bright { STAR } else { DIM };
            put(buf, inner, x0 + x, y0 + y, glyph, Style::new().fg(color));
        }
    }

    // Onda portadora: puntos que avanzan hacia el destino, o una cruz si no hay señal.
    match outage {
        Some(_) => put(
            buf,
            inner,
            (path_start + path_end) / 2,
            center,
            "✕",
            Style::new().fg(WARN).bold(),
        ),
        None => {
            for x in path_start..=path_end {
                if (f64::from(x) - t * 6.0).rem_euclid(6.0) < 1.0 {
                    put(buf, inner, x, center, "·", Style::new().fg(DIM));
                }
            }
        }
    }

    // Cuerpos celestes.
    let earth = [" ▄▄▄ ", "█████", " ▀▀▀ "];
    let dest = [" ▄▄ ", "████", " ▀▀ "];
    for (i, row) in earth.iter().enumerate() {
        let y = center - 1 + i as i32;
        put(buf, inner, earth_x, y, row, Style::new().fg(EARTH));
    }
    for (i, row) in dest.iter().enumerate() {
        let y = center - 1 + i as i32;
        put(buf, inner, dest_x, y, row, Style::new().fg(dest_color));
    }
    let label_y = y0 + sky_h;
    put(buf, inner, earth_x, label_y, "Tierra", Style::new().fg(DIM));
    put(
        buf,
        inner,
        dest_x - 1,
        label_y,
        dest_label,
        Style::new().fg(DIM),
    );

    // Paquetes en vuelo, con estela.
    for p in &app.pending {
        let progress = p.progress();
        let (leg, dir, side) = if p.returning() {
            (1.0 - (progress - 0.5) * 2.0, -1.0, 1.0)
        } else {
            (progress * 2.0, 1.0, -1.0)
        };
        let (glyph, color) = match (outage.is_some(), p.returning()) {
            (true, _) => ("◆", WARN),
            (false, false) => ("◆", GOLD),
            (false, true) => ("◇", OK),
        };
        for k in (1..=3).rev() {
            let r = (leg - dir * 0.05 * f64::from(k)).clamp(0.0, 1.0);
            let x = path_start + (r * path_len) as i32;
            let y = center + (side * 1.3 * (PI * r).sin()).round() as i32;
            let c = if k == 1 { color } else { DIM };
            put(buf, inner, x, y, "·", Style::new().fg(c));
        }
        let x = path_start + (leg * path_len) as i32;
        let y = center + (side * 1.3 * (PI * leg).sin()).round() as i32;
        put(buf, inner, x, y, glyph, Style::new().fg(color).bold());
    }

    // Línea de estado.
    let status_y = y0 + sky_h + 1;
    let (text, color) = if let Some(notice) = app.active_notice() {
        (notice.to_string(), GOLD)
    } else if let Some(o) = outage {
        (
            format!(
                "⚠ sin señal ({}) · reintentando",
                o.kind.label(app.link.target)
            ),
            WARN,
        )
    } else if let Some(p) = app.pending.first() {
        let dir = if p.returning() {
            "← regresando a la Tierra".to_string()
        } else {
            format!("→ viajando a {}", app.link.target.destination())
        };
        let extra = match app.pending.len() {
            1 => String::new(),
            n => format!(" (+{} en vuelo)", n - 1),
        };
        (
            format!("{dir} · {} restantes{extra}", fmt::dur(p.remaining())),
            ACCENT,
        )
    } else {
        (
            format!(
                "Enlace en reposo · {} km hasta {}",
                fmt::km(app.link.distance_km(now)),
                app.link.target.destination()
            ),
            DIM,
        )
    };
    put(buf, inner, x0 + 1, status_y, &text, Style::new().fg(color));
}

fn render_log(frame: &mut Frame, app: &App, area: Rect) {
    let visible = area.height.saturating_sub(2) as usize;
    let start = app.log.len().saturating_sub(visible);
    let lines: Vec<Line> = app.log[start..]
        .iter()
        .map(|l| {
            let color = if l.starts_with('⚠') {
                WARN
            } else if l.starts_with('✔') {
                OK
            } else {
                Color::Reset
            };
            Line::styled(format!("› {l}"), Style::new().fg(color))
        })
        .collect();
    frame.render_widget(Paragraph::new(lines).block(block("Registro", false)), area);
}

fn render_section(frame: &mut Frame, app: &App, area: Rect) {
    let item = app.selected_item();
    let active = app.focus == Focus::Content;
    let b = block(item.label(), active);
    let mut inner = b.inner(area);
    frame.render_widget(b, area);

    // Transición de entrada: el contenido se va desplegando de arriba abajo.
    let reveal = (app.section_since.elapsed().as_secs_f64() / WIPE_SECS).min(1.0);
    inner.height = ((f64::from(inner.height) * reveal).ceil() as u16).clamp(1, inner.height.max(1));

    match item {
        MenuItem::Correo => render_mail(frame, app, inner),
        MenuItem::Archivos => render_files(frame, app, inner),
        MenuItem::Telemetria => render_telemetry(frame, app, inner),
        MenuItem::Operador => render_operator(frame, app, inner),
        MenuItem::Ping => render_ping(frame, app, inner),
        MenuItem::Galeria => render_gallery(frame, app, inner),
    }
}

/// Mensaje mostrado mientras la sección aún no ha recibido datos.
fn render_placeholder(frame: &mut Frame, app: &App, area: Rect) {
    let hint = if app.focus == Focus::Menu {
        "Pulsa Enter para conectar."
    } else {
        "Esperando al servidor lunar..."
    };
    let lines = vec![
        Line::raw(app.selected_item().intro()),
        Line::raw(""),
        Line::styled(hint, Style::new().fg(GOLD)),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
}

fn selection_style(app: &App) -> Style {
    if app.focus == Focus::Content {
        Style::new()
            .fg(Color::Black)
            .bg(ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(ACCENT)
    }
}

fn render_mail(frame: &mut Frame, app: &App, area: Rect) {
    if let Some((from, subject, body)) = &app.mail.open {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("De: ", Style::new().fg(DIM)),
                Span::raw(from.as_str()),
            ]),
            Line::from(vec![
                Span::styled("Asunto: ", Style::new().fg(DIM)),
                Span::styled(subject.as_str(), Style::new().bold()),
            ]),
            Line::raw(""),
        ];
        lines.extend(typed(body.trim(), app.typing_since).lines().map(Line::raw));
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
        return;
    }
    let Some(headers) = &app.mail.headers else {
        return render_placeholder(frame, app, area);
    };
    let items: Vec<ListItem> = headers
        .iter()
        .map(|h| {
            ListItem::new(vec![
                Line::styled(h.subject.as_str(), Style::new().bold()),
                Line::styled(format!("  {}", h.from), Style::new().fg(DIM)),
            ])
        })
        .collect();
    let list = List::new(items)
        .highlight_symbol("▶")
        .highlight_style(selection_style(app));
    let mut state = ListState::default().with_selected(Some(app.mail.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_files(frame: &mut Frame, app: &App, area: Rect) {
    let Some(files) = &app.files.list else {
        return render_placeholder(frame, app, area);
    };
    let [list_area, download_area] = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(if app.files.download.is_some() { 3 } else { 0 }),
    ])
    .areas(area);

    let items: Vec<ListItem> = files
        .iter()
        .map(|f| {
            let done = app.files.received.contains(&f.name);
            let mark = if done {
                Span::styled(" ✔", Style::new().fg(OK))
            } else {
                Span::raw("")
            };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(f.name.as_str(), Style::new().bold()),
                    Span::styled(format!("  {} KB", f.size_kb), Style::new().fg(GOLD)),
                    mark,
                ]),
                Line::styled(format!("  {}", f.description), Style::new().fg(DIM)),
            ])
        })
        .collect();
    let list = List::new(items)
        .highlight_symbol("▶")
        .highlight_style(selection_style(app));
    let mut state = ListState::default().with_selected(Some(app.files.selected));
    frame.render_stateful_widget(list, list_area, &mut state);

    if let Some(d) = &app.files.download {
        let ratio = if d.total == 0 {
            0.0
        } else {
            d.done as f64 / d.total as f64
        };
        let gauge = Gauge::default()
            .block(
                Block::bordered()
                    .border_style(Style::new().fg(DIM))
                    .title(" Descargando "),
            )
            .gauge_style(Style::new().fg(OK).bg(TRACK))
            .ratio(ratio)
            .label(format!(
                "{} · {}/{} KB",
                d.name,
                d.done / 1024,
                d.total / 1024
            ));
        frame.render_widget(gauge, download_area);
    }
}

/// Una fila con etiqueta y minigráfica de la evolución de un valor.
fn labeled_spark(frame: &mut Frame, area: Rect, label: &str, data: Vec<u64>, color: Color) {
    let [label_area, spark_area] =
        Layout::horizontal([Constraint::Length(14), Constraint::Min(4)]).areas(area);
    frame.render_widget(
        Paragraph::new(label.to_string()).style(Style::new().fg(DIM)),
        label_area,
    );
    frame.render_widget(
        Sparkline::default()
            .data(data)
            .style(Style::new().fg(color)),
        spark_area,
    );
}

fn render_telemetry(frame: &mut Frame, app: &App, area: Rect) {
    let Some((r, received)) = &app.telemetry else {
        return render_placeholder(frame, app, area);
    };
    // El dato salió de la base hace un trayecto de ida (simulado) más lo que lleva en pantalla.
    let age = received.elapsed().mul_f64(app.link.speed) + app.link.one_way();
    let row = |label: &str, value: String, ratio: f64, color: Color| {
        Line::from(vec![
            Span::styled(format!("{label:<14}"), Style::new().fg(DIM)),
            Span::styled(bar(ratio, 20), Style::new().fg(color)),
            Span::raw(format!("  {value}")),
        ])
    };
    let lines = vec![
        Line::from(vec![Span::styled(
            if r.sun_up {
                "☀ Día lunar"
            } else {
                "☾ Noche lunar"
            },
            Style::new().fg(if r.sun_up { GOLD } else { ACCENT }).bold(),
        )]),
        Line::raw(""),
        row(
            "Temperatura",
            format!("{:.0} °C", r.temperature_c),
            (r.temperature_c + 170.0) / 300.0,
            WARN,
        ),
        row(
            "Energía solar",
            format!("{:.0} W", r.solar_w),
            r.solar_w / 480.0,
            GOLD,
        ),
        row(
            "Batería",
            format!("{:.0} %", r.battery_pct),
            r.battery_pct / 100.0,
            OK,
        ),
        row(
            "Polvo",
            format!("{} µg/m³", fmt::dec(r.dust_ugm3, 1)),
            r.dust_ugm3 / 30.0,
            ACCENT,
        ),
        row(
            "Radiación",
            format!("{} µSv/h", fmt::dec(r.radiation_usv_h, 2)),
            r.radiation_usv_h / 0.6,
            ACCENT,
        ),
        Line::raw(""),
        Line::styled(
            format!(
                "Lectura tomada hace {} (lo que tarda la señal en llegar).",
                fmt::dur(age)
            ),
            Style::new().fg(DIM),
        ),
    ];
    let [text_area, history_area] =
        Layout::vertical([Constraint::Length(lines.len() as u16), Constraint::Min(0)]).areas(area);
    frame.render_widget(Paragraph::new(lines), text_area);

    if history_area.height >= 3 {
        let [_, temp_row, solar_row] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(history_area);
        let history = &app.telemetry_history;
        let temp = history
            .iter()
            .map(|h| (h.temperature_c + 170.0) as u64)
            .collect();
        let solar = history.iter().map(|h| h.solar_w as u64).collect();
        labeled_spark(frame, temp_row, "Temp. reciente", temp, WARN);
        labeled_spark(frame, solar_row, "Sol reciente", solar, GOLD);
    }
}

fn render_operator(frame: &mut Frame, app: &App, area: Rect) {
    let op = &app.operator;
    if op.history.is_empty() {
        return render_placeholder(frame, app, area);
    }
    let [chat, options, input] = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(op.options.len().min(6) as u16),
        Constraint::Length(3),
    ])
    .areas(area);

    let last = op.history.len() - 1;
    let lines: Vec<Line> = op
        .history
        .iter()
        .enumerate()
        .flat_map(|(i, (user, text))| {
            let (who, color) = if *user {
                ("Tú", ACCENT)
            } else {
                ("Operador", GOLD)
            };
            // La última respuesta del operador se escribe letra a letra.
            let shown = if i == last && !*user {
                typed(text, app.typing_since)
            } else {
                text.as_str()
            };
            [
                Line::from(vec![
                    Span::styled(format!("{who}: "), Style::new().fg(color).bold()),
                    Span::raw(shown),
                ]),
                Line::raw(""),
            ]
        })
        .collect();
    // Estimación de líneas tras el ajuste de ancho, para mostrar siempre el final.
    let width = chat.width.max(1) as usize;
    let total: usize = op
        .history
        .iter()
        .map(|(_, t)| (t.chars().count() + 12).div_ceil(width) + 1)
        .sum();
    let scroll = total.saturating_sub(chat.height as usize) as u16;
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        chat,
    );

    let items: Vec<ListItem> = op
        .options
        .iter()
        .map(|o| ListItem::new(o.label.as_str()))
        .collect();
    let list = List::new(items)
        .highlight_symbol("▶")
        .highlight_style(selection_style(app));
    let mut state = ListState::default().with_selected(Some(op.selected));
    frame.render_stateful_widget(list, options, &mut state);

    let cursor = if app.focus == Focus::Content {
        "▌"
    } else {
        ""
    };
    frame.render_widget(
        Paragraph::new(format!("{}{cursor}", op.input)).block(
            Block::bordered()
                .border_style(Style::new().fg(DIM))
                .title(" Escribe una palabra clave "),
        ),
        input,
    );
}

fn render_ping(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![Line::raw(app.selected_item().intro()), Line::raw("")];

    if app.ping.rtts.is_empty() {
        lines.push(Line::styled(
            "Aún no has lanzado ningún ping.",
            Style::new().fg(DIM),
        ));
    } else {
        for (i, rtt) in app.ping.rtts.iter().enumerate().rev().take(5).rev() {
            lines.push(Line::from(vec![
                Span::styled(format!("#{:<3}", i + 1), Style::new().fg(DIM)),
                Span::raw(format!(
                    "Base Tranquilidad: {}",
                    fmt::dur(Duration::from_secs_f64(*rtt))
                )),
            ]));
        }
    }

    if let Some(route) = &app.ping.route {
        lines.push(Line::raw(""));
        lines.push(Line::styled("Ruta:", Style::new().fg(ACCENT).bold()));
        let one_way = app.link.one_way();
        for (i, hop) in route.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(format!("{:>2}  ", i + 1), Style::new().fg(DIM)),
                Span::raw(format!("{:<34}", hop.name)),
                Span::styled(
                    format!("{:>12}", fmt::latency(one_way.mul_f64(hop.fraction))),
                    Style::new().fg(GOLD),
                ),
            ]));
        }
    }

    let with_history = app.ping.rtts.len() >= 2;
    let [text_area, spark_area] = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(u16::from(with_history)),
    ])
    .areas(area);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), text_area);
    if with_history {
        let data = app.ping.rtts.iter().map(|r| (r * 100.0) as u64).collect();
        labeled_spark(frame, spark_area, "Latencia", data, ACCENT);
    }
}

fn render_gallery(frame: &mut Frame, app: &App, area: Rect) {
    let Some((index, total, title, art)) = &app.gallery.image else {
        return render_placeholder(frame, app, area);
    };
    let mut lines = vec![
        Line::styled(
            format!("{title}  ({}/{total})", index + 1),
            Style::new().fg(ACCENT).bold(),
        ),
        Line::raw(""),
    ];
    lines.extend(art.lines().map(|l| Line::styled(l, Style::new().fg(GOLD))));
    frame.render_widget(Paragraph::new(lines), area);
}
