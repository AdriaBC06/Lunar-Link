//! Renderizado de la TUI.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Gauge, List, ListItem, ListState, Paragraph};

use crate::app::{App, MenuItem};

const ACCENT: Color = Color::Rgb(180, 190, 254);
const DIM: Color = Color::Rgb(108, 112, 134);

pub const MENU_WIDTH: u16 = 24;

pub fn render(frame: &mut Frame, app: &App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let [menu, main] =
        Layout::horizontal([Constraint::Length(MENU_WIDTH), Constraint::Min(20)]).areas(body);

    render_status(frame, app, header);
    render_menu(frame, app, menu);
    render_main(frame, app, main);
    frame.render_widget(
        Paragraph::new(" ↑↓ navegar · Enter seleccionar · clic con el ratón · q salir")
            .style(Style::new().fg(DIM)),
        footer,
    );
}

/// Área interior de la lista del menú (para traducir clics en opciones).
pub fn menu_inner(frame_area: Rect) -> Rect {
    let [_, body, _] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .areas(frame_area);
    let [menu, _] =
        Layout::horizontal([Constraint::Length(MENU_WIDTH), Constraint::Min(20)]).areas(body);
    block("Menú").inner(menu)
}

fn block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM))
        .title(Span::styled(
            format!(" {title} "),
            Style::new().fg(ACCENT).bold(),
        ))
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let rtt = app.link.round_trip().as_secs_f64();
    let line = Line::from(vec![
        Span::styled("🌙 LUNAR LINK", Style::new().fg(ACCENT).bold()),
        Span::raw("  "),
        Span::styled("●", Style::new().fg(Color::Green)),
        Span::raw(format!(" enlace con {}", app.link.target.label())),
        Span::styled("  │  ", Style::new().fg(DIM)),
        Span::raw(format!("ida y vuelta {rtt:.2} s")),
    ]);
    frame.render_widget(Paragraph::new(line).block(block("Estado")), area);
}

fn render_menu(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = MenuItem::ALL
        .iter()
        .map(|i| ListItem::new(format!(" {}", i.label())))
        .collect();
    let list = List::new(items)
        .block(block("Menú"))
        .highlight_symbol("▶")
        .highlight_style(
            Style::new()
                .fg(Color::Black)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        );
    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    let [flight, log] = Layout::vertical([Constraint::Length(5), Constraint::Min(3)]).areas(area);

    let flight_block = block("Petición en vuelo");
    match &app.in_flight {
        Some(f) => {
            let direction = if f.returning() {
                "← regresando a la Tierra"
            } else {
                "→ viajando a la Luna"
            };
            let label = format!(
                "{direction} · {:.1} s restantes",
                f.remaining().as_secs_f64()
            );
            let gauge = Gauge::default()
                .block(flight_block)
                .gauge_style(Style::new().fg(ACCENT).bg(Color::Rgb(49, 50, 68)))
                .ratio(f.progress())
                .label(label);
            frame.render_widget(gauge, flight);
        }
        None => frame.render_widget(
            Paragraph::new("Sin peticiones en vuelo.")
                .style(Style::new().fg(DIM))
                .block(flight_block),
            flight,
        ),
    }

    let visible = log.height.saturating_sub(2) as usize;
    let start = app.log.len().saturating_sub(visible);
    let lines: Vec<Line> = app.log[start..]
        .iter()
        .map(|l| Line::from(format!("› {l}")))
        .collect();
    frame.render_widget(Paragraph::new(lines).block(block("Registro")), log);
}
