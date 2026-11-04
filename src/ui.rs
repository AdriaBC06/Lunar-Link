//! Renderizado de la TUI.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Gauge, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::{App, Focus, MenuItem};

const ACCENT: Color = Color::Rgb(180, 190, 254);
const DIM: Color = Color::Rgb(108, 112, 134);
const OK: Color = Color::Rgb(166, 227, 161);
const WARN: Color = Color::Rgb(243, 139, 168);
const GOLD: Color = Color::Rgb(249, 226, 175);
const TRACK: Color = Color::Rgb(49, 50, 68);

pub const MENU_WIDTH: u16 = 24;

fn areas(area: Rect) -> [Rect; 3] {
    Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .areas(area)
}

pub fn render(frame: &mut Frame, app: &App) {
    let [header, body, footer] = areas(frame.area());
    let [menu, main] =
        Layout::horizontal([Constraint::Length(MENU_WIDTH), Constraint::Min(20)]).areas(body);
    let [flight, section, log] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(6),
    ])
    .areas(main);

    render_status(frame, app, header);
    render_menu(frame, app, menu);
    render_flight(frame, app, flight);
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

/// Separa los miles con puntos: 384512 → "384.512".
fn fmt_km(km: f64) -> String {
    let digits = (km.round() as u64).to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}

fn bar(ratio: f64, width: usize) -> String {
    let filled = (ratio.clamp(0.0, 1.0) * width as f64).round() as usize;
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let t = app.link.elapsed();
    let mut spans = vec![
        Span::styled("🌙 LUNAR LINK", Style::new().fg(ACCENT).bold()),
        Span::styled("  │  ", Style::new().fg(DIM)),
    ];
    match app.link.status_at(t) {
        Some(outage) => {
            let left = outage.ends_at.saturating_sub(t).as_secs_f64();
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
                Span::raw(format!("{} km", fmt_km(app.link.distance_km(t)))),
                Span::styled("  │  ", Style::new().fg(DIM)),
                Span::raw(format!(
                    "ida y vuelta {:.2} s",
                    app.link.round_trip().as_secs_f64()
                )),
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

fn render_flight(frame: &mut Frame, app: &App, area: Rect) {
    let b = block("Petición en vuelo", false);
    let Some(f) = &app.in_flight else {
        frame.render_widget(
            Paragraph::new("Sin peticiones en vuelo.")
                .style(Style::new().fg(DIM))
                .block(b),
            area,
        );
        return;
    };
    let target = app.link.target.destination();
    let label = match app.link.status_at(app.link.elapsed()) {
        Some(o) => format!(
            "⚠ sin señal ({}) · reintentando",
            o.kind.label(app.link.target)
        ),
        None => {
            let dir = if f.returning() {
                "← regresando a la Tierra".to_string()
            } else {
                format!("→ viajando a {target}")
            };
            format!("{dir} · {:.1} s restantes", f.remaining().as_secs_f64())
        }
    };
    let gauge = Gauge::default()
        .block(b)
        .gauge_style(Style::new().fg(ACCENT).bg(TRACK))
        .ratio(f.progress())
        .label(label);
    frame.render_widget(gauge, area);
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
    let inner = b.inner(area);
    frame.render_widget(b, area);

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
        lines.extend(body.trim().lines().map(Line::raw));
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

fn render_telemetry(frame: &mut Frame, app: &App, area: Rect) {
    let Some((r, received)) = &app.telemetry else {
        return render_placeholder(frame, app, area);
    };
    let age = received.elapsed() + app.link.one_way();
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
            format!("{:.1} µg/m³", r.dust_ugm3),
            r.dust_ugm3 / 30.0,
            ACCENT,
        ),
        row(
            "Radiación",
            format!("{:.2} µSv/h", r.radiation_usv_h),
            r.radiation_usv_h / 0.6,
            ACCENT,
        ),
        Line::raw(""),
        Line::styled(
            format!(
                "Lectura con {:.1} s de antigüedad (el dato ya viajó hasta ti).",
                age.as_secs_f64()
            ),
            Style::new().fg(DIM),
        ),
    ];
    frame.render_widget(Paragraph::new(lines), area);
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

    let lines: Vec<Line> = op
        .history
        .iter()
        .flat_map(|(user, text)| {
            let (who, color) = if *user {
                ("Tú", ACCENT)
            } else {
                ("Operador", GOLD)
            };
            [
                Line::from(vec![
                    Span::styled(format!("{who}: "), Style::new().fg(color).bold()),
                    Span::raw(text.as_str()),
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
                Span::raw(format!("Base Tranquilidad: {rtt:.2} s")),
            ]));
        }
    }

    if let Some(route) = &app.ping.route {
        lines.push(Line::raw(""));
        lines.push(Line::styled("Ruta:", Style::new().fg(ACCENT).bold()));
        let one_way = app.link.one_way().as_secs_f64();
        for (i, hop) in route.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(format!("{:>2}  ", i + 1), Style::new().fg(DIM)),
                Span::raw(format!("{:<34}", hop.name)),
                Span::styled(
                    format!("{:>8.1} ms", hop.fraction * one_way * 1000.0),
                    Style::new().fg(GOLD),
                ),
            ]));
        }
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
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
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}
