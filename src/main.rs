use std::io;
use std::time::Duration;

use clap::Parser;
use lunar_link::app::App;
use lunar_link::link::{Link, Target};
use lunar_link::ui;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use tokio::sync::mpsc;

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
    let link = Link {
        target: cli.target,
        fixed_one_way: cli.delay.map(Duration::from_secs_f64),
        outages: !cli.no_outages,
    };

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
        while let Ok(response) = rx.try_recv() {
            app.on_response(response);
        }
        terminal.draw(|frame| ui::render(frame, &app))?;

        if !event::poll(Duration::from_millis(33))? {
            continue;
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
                KeyCode::Enter => dispatch(&mut app, &tx),
                _ => {}
            },
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                let inner = ui::menu_inner(terminal.get_frame().area());
                let inside = mouse.column >= inner.x
                    && mouse.column < inner.x + inner.width
                    && mouse.row >= inner.y;
                if inside {
                    app.select((mouse.row - inner.y) as usize);
                    dispatch(&mut app, &tx);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn dispatch(app: &mut App, tx: &mpsc::UnboundedSender<lunar_link::server::Response>) {
    if let Some(request) = app.activate() {
        let link = app.link;
        let tx = tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(link.send(request).await);
        });
    }
}
