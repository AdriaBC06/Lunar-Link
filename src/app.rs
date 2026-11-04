//! Estado de la aplicación y manejo de entrada.

use std::time::Duration;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::time::Instant;

use crate::content::OptionEntry;
use crate::link::{Link, LinkEvent, OutageKind};
use crate::server::{self, FileInfo, Hop, MailHeader, OperatorInput, Reading, Request, Response};

const MAX_LOG: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Correo,
    Archivos,
    Telemetria,
    Operador,
    Ping,
    Galeria,
}

impl MenuItem {
    pub const ALL: [MenuItem; 6] = [
        MenuItem::Correo,
        MenuItem::Archivos,
        MenuItem::Telemetria,
        MenuItem::Operador,
        MenuItem::Ping,
        MenuItem::Galeria,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MenuItem::Correo => "Correo",
            MenuItem::Archivos => "Archivos",
            MenuItem::Telemetria => "Telemetría",
            MenuItem::Operador => "Operador",
            MenuItem::Ping => "Ping",
            MenuItem::Galeria => "Galería",
        }
    }

    /// Texto mostrado antes de conectar con la sección.
    pub fn intro(self) -> &'static str {
        match self {
            MenuItem::Correo => "Buzón de la tripulación de Base Tranquilidad.",
            MenuItem::Archivos => {
                "Archivos del servidor. Las descargas dependen del ancho de banda."
            }
            MenuItem::Telemetria => "Datos en vivo de la base. Llegan con el retardo del enlace.",
            MenuItem::Operador => {
                "Habla con el operador: elige una opción o escribe una palabra clave."
            }
            MenuItem::Ping => "Mide tu enlace (Enter) o traza la ruta hasta la Luna (t).",
            MenuItem::Galeria => "Imágenes en ASCII de la superficie lunar.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Menu,
    Content,
}

/// Petición del usuario que ya salió y aún no ha vuelto.
#[derive(Debug, Clone, Copy)]
pub struct InFlight {
    pub sent_at: Instant,
    pub round_trip: Duration,
}

impl InFlight {
    pub fn elapsed(&self) -> Duration {
        self.sent_at.elapsed()
    }

    /// Avance estimado del viaje; no llega al 100% hasta recibir la respuesta.
    pub fn progress(&self) -> f64 {
        (self.elapsed().as_secs_f64() / self.round_trip.as_secs_f64()).clamp(0.0, 0.99)
    }

    /// `true` si ya va de vuelta hacia la Tierra.
    pub fn returning(&self) -> bool {
        self.progress() >= 0.5
    }

    pub fn remaining(&self) -> Duration {
        self.round_trip.saturating_sub(self.elapsed())
    }
}

#[derive(Debug, Default)]
pub struct MailState {
    pub headers: Option<Vec<MailHeader>>,
    pub selected: usize,
    pub open: Option<(String, String, String)>,
}

#[derive(Debug)]
pub struct Download {
    pub name: String,
    pub done: u64,
    pub total: u64,
}

#[derive(Debug, Default)]
pub struct FilesState {
    pub list: Option<Vec<FileInfo>>,
    pub selected: usize,
    pub download: Option<Download>,
    pub received: Vec<String>,
}

#[derive(Debug, Default)]
pub struct OperatorState {
    pub options: Vec<OptionEntry>,
    pub selected: usize,
    pub input: String,
    /// Conversación: `true` si habla el usuario.
    pub history: Vec<(bool, String)>,
}

#[derive(Debug, Default)]
pub struct PingState {
    pub rtts: Vec<f64>,
    pub route: Option<Vec<Hop>>,
}

#[derive(Debug, Default)]
pub struct GalleryState {
    pub index: usize,
    pub image: Option<(usize, usize, String, String)>,
}

pub struct App {
    pub link: Link,
    pub focus: Focus,
    pub selected: usize,
    pub in_flight: Option<InFlight>,
    pub log: Vec<String>,
    pub should_quit: bool,
    pub mail: MailState,
    pub files: FilesState,
    pub telemetry: Option<(Reading, Instant)>,
    telemetry_pending: bool,
    pub operator: OperatorState,
    pub ping: PingState,
    pub gallery: GalleryState,
    last_outage: Option<OutageKind>,
}

impl App {
    pub fn new(link: Link) -> Self {
        Self {
            link,
            focus: Focus::Menu,
            selected: 0,
            in_flight: None,
            log: vec!["Enlace establecido con la Tierra. Esperando órdenes.".into()],
            should_quit: false,
            mail: MailState::default(),
            files: FilesState::default(),
            telemetry: None,
            telemetry_pending: false,
            operator: OperatorState::default(),
            ping: PingState::default(),
            gallery: GalleryState::default(),
            last_outage: None,
        }
    }

    pub fn selected_item(&self) -> MenuItem {
        MenuItem::ALL[self.selected]
    }

    fn log(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > MAX_LOG {
            self.log.remove(0);
        }
    }

    /// Registra los cambios de estado de la señal. Se llama en cada fotograma.
    pub fn tick(&mut self) {
        let now = self.link.status_at(self.link.elapsed()).map(|o| o.kind);
        if now != self.last_outage {
            match now {
                Some(kind) => self.log(format!(
                    "⚠ Pérdida de señal ({}). Reintentando en la siguiente ventana.",
                    kind.label(self.link.target)
                )),
                None => self.log("✔ Señal recuperada."),
            }
            self.last_outage = now;
        }
    }

    /// Lecturas automáticas de la telemetría mientras la pantalla está abierta.
    pub fn poll(&mut self) -> Option<Request> {
        let watching = self.focus == Focus::Content && self.selected_item() == MenuItem::Telemetria;
        if watching && !self.telemetry_pending {
            self.telemetry_pending = true;
            return Some(Request::Telemetry);
        }
        None
    }

    /// Marca la petición como enviada. Devuelve `None` si ya hay otra en vuelo.
    fn begin(&mut self, request: Request, log: &str) -> Option<Request> {
        if self.in_flight.is_some() {
            self.log("Ya hay una petición en vuelo.");
            return None;
        }
        let mut round_trip = self.link.round_trip();
        if let Request::Download(i) = &request
            && let Some(bytes) = server::file_size_bytes(*i)
        {
            round_trip += Duration::from_secs_f64(bytes as f64 / self.link.bandwidth as f64);
        }
        self.in_flight = Some(InFlight {
            sent_at: Instant::now(),
            round_trip,
        });
        self.log(log);
        Some(request)
    }

    // ---- Entrada ----

    pub fn on_key(&mut self, key: KeyEvent) -> Option<Request> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return None;
        }
        match self.focus {
            Focus::Menu => self.menu_key(key.code),
            Focus::Content => self.content_key(key.code),
        }
    }

    pub fn select(&mut self, index: usize) {
        if index < MenuItem::ALL.len() {
            self.selected = index;
        }
    }

    fn menu_key(&mut self, code: KeyCode) -> Option<Request> {
        let n = MenuItem::ALL.len();
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => self.selected = (self.selected + 1) % n,
            KeyCode::Up | KeyCode::Char('k') => self.selected = (self.selected + n - 1) % n,
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => return self.open_section(),
            _ => {}
        }
        None
    }

    /// Entra en la sección seleccionada y pide sus datos la primera vez.
    pub fn open_section(&mut self) -> Option<Request> {
        self.focus = Focus::Content;
        match self.selected_item() {
            MenuItem::Correo if self.mail.headers.is_none() => {
                self.begin(Request::ListMail, "Abriendo el buzón de correo...")
            }
            MenuItem::Archivos if self.files.list.is_none() => {
                self.begin(Request::ListFiles, "Pidiendo la lista de archivos...")
            }
            MenuItem::Operador if self.operator.history.is_empty() => self.begin(
                Request::Operator(OperatorInput::Goto("inicio".into())),
                "Llamando al operador...",
            ),
            MenuItem::Galeria if self.gallery.image.is_none() => {
                self.begin(Request::Gallery(0), "Descargando imagen...")
            }
            _ => None,
        }
    }

    fn content_key(&mut self, code: KeyCode) -> Option<Request> {
        let item = self.selected_item();
        if item == MenuItem::Operador {
            return self.operator_key(code);
        }
        if item == MenuItem::Galeria
            && matches!(
                code,
                KeyCode::Left | KeyCode::Right | KeyCode::Char('h' | 'l')
            )
        {
            return self.gallery_step(matches!(code, KeyCode::Right | KeyCode::Char('l')));
        }
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => {
                if item == MenuItem::Correo && self.mail.open.is_some() {
                    self.mail.open = None;
                } else {
                    self.focus = Focus::Menu;
                }
            }
            _ => {
                return match item {
                    MenuItem::Correo => self.mail_key(code),
                    MenuItem::Archivos => self.files_key(code),
                    MenuItem::Ping => self.ping_key(code),
                    _ => None,
                };
            }
        }
        None
    }

    fn mail_key(&mut self, code: KeyCode) -> Option<Request> {
        if self.mail.open.is_some() {
            return None;
        }
        let len = self.mail.headers.as_ref()?.len();
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.mail.selected = (self.mail.selected + 1) % len
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.mail.selected = (self.mail.selected + len - 1) % len
            }
            KeyCode::Enter => {
                return self.begin(Request::ReadMail(self.mail.selected), "Abriendo mensaje...");
            }
            _ => {}
        }
        None
    }

    fn files_key(&mut self, code: KeyCode) -> Option<Request> {
        let len = self.files.list.as_ref()?.len();
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.files.selected = (self.files.selected + 1) % len
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.files.selected = (self.files.selected + len - 1) % len
            }
            KeyCode::Enter if self.files.download.is_none() => {
                let file = self.files.list.as_ref()?.get(self.files.selected)?.clone();
                let request = self.begin(
                    Request::Download(self.files.selected),
                    &format!("Solicitando {}...", file.name),
                )?;
                self.files.download = Some(Download {
                    name: file.name,
                    done: 0,
                    total: u64::from(file.size_kb) * 1024,
                });
                return Some(request);
            }
            _ => {}
        }
        None
    }

    fn ping_key(&mut self, code: KeyCode) -> Option<Request> {
        match code {
            KeyCode::Enter => self.begin(Request::Ping, "Ping enviado..."),
            KeyCode::Char('t') => self.begin(Request::Traceroute, "Trazando la ruta..."),
            _ => None,
        }
    }

    fn gallery_step(&mut self, forward: bool) -> Option<Request> {
        let (_, total, _, _) = self.gallery.image.as_ref()?;
        let total = *total;
        let next = if forward {
            (self.gallery.index + 1) % total
        } else {
            (self.gallery.index + total - 1) % total
        };
        let request = self.begin(Request::Gallery(next), "Descargando imagen...")?;
        self.gallery.index = next;
        Some(request)
    }

    fn operator_key(&mut self, code: KeyCode) -> Option<Request> {
        let op = &mut self.operator;
        match code {
            KeyCode::Char(c) => op.input.push(c),
            KeyCode::Backspace => {
                op.input.pop();
            }
            KeyCode::Esc => {
                if op.input.is_empty() {
                    self.focus = Focus::Menu;
                } else {
                    op.input.clear();
                }
            }
            KeyCode::Down if !op.options.is_empty() => {
                op.selected = (op.selected + 1) % op.options.len();
            }
            KeyCode::Up if !op.options.is_empty() => {
                op.selected = (op.selected + op.options.len() - 1) % op.options.len();
            }
            KeyCode::Enter => {
                let typed = op.input.trim().to_string();
                let (input, said) = if !typed.is_empty() {
                    (OperatorInput::Say(typed.clone()), typed)
                } else {
                    let option = op.options.get(op.selected)?;
                    (
                        OperatorInput::Goto(option.goto.clone()),
                        option.label.clone(),
                    )
                };
                let request =
                    self.begin(Request::Operator(input), "Mensaje enviado al operador...")?;
                self.operator.input.clear();
                self.operator.history.push((true, said));
                return Some(request);
            }
            _ => {}
        }
        None
    }

    // ---- Eventos del enlace ----

    pub fn on_event(&mut self, event: LinkEvent) {
        match event {
            LinkEvent::Progress { done, total } => {
                if let Some(d) = &mut self.files.download {
                    d.done = done;
                    d.total = total;
                }
            }
            LinkEvent::Response(response) => self.on_response(response),
        }
    }

    fn on_response(&mut self, response: Response) {
        let sent_at = self.in_flight.map(|f| f.sent_at);
        if !matches!(response, Response::Telemetry(_)) {
            self.in_flight = None;
        }
        match response {
            Response::Pong { station } => {
                let rtt = sent_at
                    .map(|t| t.elapsed().as_secs_f64())
                    .unwrap_or_default();
                self.ping.rtts.push(rtt);
                self.log(format!("Respuesta de {station}: pong en {rtt:.2} s."));
            }
            Response::Route(hops) => {
                self.log("Ruta recibida.");
                self.ping.route = Some(hops);
            }
            Response::MailList(headers) => {
                self.log(format!("{} mensajes en el buzón.", headers.len()));
                self.mail.headers = Some(headers);
            }
            Response::Mail {
                from,
                subject,
                body,
            } => self.mail.open = Some((from, subject, body)),
            Response::Files(list) => {
                self.log(format!("{} archivos disponibles.", list.len()));
                self.files.list = Some(list);
            }
            Response::FileReceived { name, size_kb } => {
                self.files.download = None;
                self.log(format!("✔ Descarga completa: {name} ({size_kb} KB)."));
                self.files.received.push(name);
            }
            Response::Telemetry(reading) => {
                self.telemetry_pending = false;
                self.telemetry = Some((reading, Instant::now()));
            }
            Response::Operator { text, options } => {
                self.operator.history.push((false, text));
                self.operator.options = options;
                self.operator.selected = 0;
            }
            Response::Image {
                index,
                total,
                title,
                art,
            } => {
                self.gallery.index = index;
                self.gallery.image = Some((index, total, title, art));
            }
            Response::NotFound => self.log("El servidor no encontró lo solicitado."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::Target;
    use ratatui::crossterm::event::KeyEvent;

    fn app() -> App {
        App::new(Link::new(
            Target::Luna,
            Some(Duration::from_secs(1)),
            false,
            1,
        ))
    }

    fn press(app: &mut App, code: KeyCode) -> Option<Request> {
        app.on_key(KeyEvent::from(code))
    }

    #[tokio::test(start_paused = true)]
    async fn opening_a_section_requests_its_data_only_once() {
        let mut app = app();
        assert_eq!(press(&mut app, KeyCode::Enter), Some(Request::ListMail));
        assert_eq!(app.focus, Focus::Content);
        assert!(app.in_flight.is_some());
        // Mientras hay una petición en vuelo no se envía otra.
        assert_eq!(app.begin(Request::Ping, "x"), None);

        app.on_event(LinkEvent::Response(server::handle(
            &Request::ListMail,
            Duration::ZERO,
        )));
        assert!(app.in_flight.is_none() && app.mail.headers.is_some());
        press(&mut app, KeyCode::Esc);
        assert_eq!(
            press(&mut app, KeyCode::Enter),
            None,
            "no vuelve a pedir el buzón"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn operator_sends_typed_text_or_selected_option() {
        let mut app = app();
        app.selected = MenuItem::Operador as usize;
        press(&mut app, KeyCode::Enter);
        app.on_event(LinkEvent::Response(server::handle(
            &Request::Operator(OperatorInput::Goto("inicio".into())),
            Duration::ZERO,
        )));
        assert!(!app.operator.options.is_empty());

        // 'q' se escribe en el campo, no cierra la aplicación.
        for c in "hielo".chars() {
            press(&mut app, KeyCode::Char(c));
        }
        assert!(!app.should_quit);
        assert_eq!(
            press(&mut app, KeyCode::Enter),
            Some(Request::Operator(OperatorInput::Say("hielo".into())))
        );
    }

    #[tokio::test(start_paused = true)]
    async fn telemetry_polls_only_while_visible() {
        let mut app = app();
        assert_eq!(app.poll(), None);
        app.selected = MenuItem::Telemetria as usize;
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.poll(), Some(Request::Telemetry));
        assert_eq!(app.poll(), None, "una lectura a la vez");
        app.on_event(LinkEvent::Response(server::handle(
            &Request::Telemetry,
            Duration::ZERO,
        )));
        assert_eq!(app.poll(), Some(Request::Telemetry));
        press(&mut app, KeyCode::Esc);
        app.on_event(LinkEvent::Response(server::handle(
            &Request::Telemetry,
            Duration::ZERO,
        )));
        assert_eq!(app.poll(), None);
    }
}
