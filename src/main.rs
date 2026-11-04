use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::Parser;
use lunar_link::app::App;
use lunar_link::link::{Link, LinkEvent, Target};
use lunar_link::server::Request;
use lunar_link::ui;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseButton, MouseEventKind,
};
use ratatui::crossterm::execute;
use tokio::sync::mpsc::{self, UnboundedSender};

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
    /// Desactiva los cortes de señal
    #[arg(long)]
    no_outages: bool,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    let link = Link::new(
        cli.target,
        cli.delay.map(Duration::from_secs_f64),
        !cli.no_outages,
        seed,
    );

    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal, App::new(link)).await;
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
        if let Some(request) = app.poll() {
            dispatch(&app, request, &tx);
        }
        terminal.draw(|frame| ui::render(frame, &app))?;

        if !event::poll(Duration::from_millis(33))? {
            continue;
        }
        let request = match event::read()? {
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
        if let Some(request) = request {
            dispatch(&app, request, &tx);
        }
    }
    Ok(())
}

fn dispatch(app: &App, request: Request, tx: &UnboundedSender<LinkEvent>) {
    let link = app.link;
    let tx = tx.clone();
    tokio::spawn(async move { link.transmit(request, tx).await });
}
