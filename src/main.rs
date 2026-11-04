use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::Parser;
use lunar_link::app::{App, Outgoing};
use lunar_link::link::{Link, LinkEvent, Target};
use lunar_link::ui;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseButton, MouseEventKind,
};
use ratatui::crossterm::execute;
use tokio::sync::mpsc::{self, UnboundedSender};

/// Intervalo entre fotogramas (~60 FPS).
const FRAME: Duration = Duration::from_millis(16);

#[derive(Parser)]
#[command(
    version,
    about = "Un pequeño servidor lunar al que te conectas desde la terminal"
)]
struct Cli {
    /// Destino del enlace
    #[arg(long, value_enum, default_value = "luna")]
    target: Target,
    /// Retardo de ida fijo en segundos (sustituye al del destino)
    #[arg(long, value_name = "SEGUNDOS")]
    delay: Option<f64>,
    /// Acelera el tiempo de viaje y las descargas (útil en Marte: --speed 100)
    #[arg(long, value_name = "FACTOR", default_value_t = 1.0, value_parser = parse_speed)]
    speed: f64,
    /// Desactiva los cortes de señal
    #[arg(long)]
    no_outages: bool,
}

fn parse_speed(s: &str) -> Result<f64, String> {
    match s.parse::<f64>() {
        Ok(v) if v.is_finite() && v >= 1.0 => Ok(v),
        _ => Err("debe ser un número mayor o igual que 1".into()),
    }
}

/// `true` si la terminal declara soporte de color verdadero.
fn supports_truecolor() -> bool {
    std::env::var("COLORTERM").is_ok_and(|v| v.contains("truecolor") || v.contains("24bit"))
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    let mut link = Link::new(
        cli.target,
        cli.delay.map(Duration::from_secs_f64),
        !cli.no_outages,
        seed,
    );
    link.speed = cli.speed;

    let mut app = App::new(link);
    app.truecolor = supports_truecolor();
    if cli.target == Target::Marte {
        if cli.speed > 1.0 {
            app.log(format!(
                "Modo Marte: tiempo acelerado ×{}. Las respuestas llegan como avisos.",
                cli.speed
            ));
        } else {
            app.log("Modo Marte: las respuestas tardan minutos. Prueba --speed 100 para acelerar.");
        }
    }

    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal, app).await;
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

async fn run(terminal: &mut ratatui::DefaultTerminal, mut app: App) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    while !app.should_quit {
        while let Ok(event) = rx.try_recv() {
            app.on_event(event);
        }
        app.tick();
        if let Some(out) = app.poll() {
            dispatch(&app, out, &tx);
        }
        terminal.draw(|frame| ui::render(frame, &app))?;

        if !event::poll(FRAME)? {
            continue;
        }
        let out = match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => app.on_key(key),
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                let inner = ui::menu_inner(terminal.get_frame().area());
                let inside = mouse.column >= inner.x
                    && mouse.column < inner.x + inner.width
                    && mouse.row >= inner.y;
                if inside {
                    app.select((mouse.row - inner.y) as usize);
                    app.open_section()
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(out) = out {
            dispatch(&app, out, &tx);
        }
    }
    Ok(())
}

fn dispatch(app: &App, out: Outgoing, tx: &UnboundedSender<LinkEvent>) {
    let link = app.link;
    let tx = tx.clone();
    tokio::spawn(async move { link.transmit(out.id, out.request, tx).await });
}
