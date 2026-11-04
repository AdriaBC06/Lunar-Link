//! Estado de la aplicación.

use std::time::Duration;

use tokio::time::Instant;

use crate::link::Link;
use crate::server::{Request, Response};

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
}

/// Petición que ya salió y aún no ha vuelto.
#[derive(Debug, Clone, Copy)]
pub struct InFlight {
    pub sent_at: Instant,
    pub round_trip: Duration,
}

impl InFlight {
    pub fn elapsed(&self) -> Duration {
        self.sent_at.elapsed().min(self.round_trip)
    }

    pub fn progress(&self) -> f64 {
        (self.elapsed().as_secs_f64() / self.round_trip.as_secs_f64()).clamp(0.0, 1.0)
    }

    /// Fase del viaje: `true` si ya va de vuelta hacia la Tierra.
    pub fn returning(&self) -> bool {
        self.progress() >= 0.5
    }

    pub fn remaining(&self) -> Duration {
        self.round_trip.saturating_sub(self.elapsed())
    }
}

pub struct App {
    pub link: Link,
    pub selected: usize,
    pub in_flight: Option<InFlight>,
    pub log: Vec<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(link: Link) -> Self {
        Self {
            link,
            selected: 0,
            in_flight: None,
            log: vec!["Enlace establecido con la Tierra. Esperando órdenes.".into()],
            should_quit: false,
        }
    }

    pub fn selected_item(&self) -> MenuItem {
        MenuItem::ALL[self.selected]
    }

    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1) % MenuItem::ALL.len();
    }

    pub fn select_prev(&mut self) {
        self.selected = (self.selected + MenuItem::ALL.len() - 1) % MenuItem::ALL.len();
    }

    pub fn select(&mut self, index: usize) {
        if index < MenuItem::ALL.len() {
            self.selected = index;
        }
    }

    /// Activa la opción seleccionada. Devuelve la petición a enviar, si la hay.
    pub fn activate(&mut self) -> Option<Request> {
        match self.selected_item() {
            MenuItem::Ping => {
                if self.in_flight.is_some() {
                    self.log.push("Ya hay una petición en vuelo.".into());
                    return None;
                }
                self.in_flight = Some(InFlight {
                    sent_at: Instant::now(),
                    round_trip: self.link.round_trip(),
                });
                self.log.push("Ping enviado hacia la Luna...".into());
                Some(Request::Ping)
            }
            other => {
                self.log
                    .push(format!("{}: disponible en la Fase 2.", other.label()));
                None
            }
        }
    }

    pub fn on_response(&mut self, response: Response) {
        let rtt = self.in_flight.take().map(|f| f.sent_at.elapsed());
        match response {
            Response::Pong { station } => {
                let rtt = rtt
                    .map(|d| format!("{:.2} s", d.as_secs_f64()))
                    .unwrap_or_default();
                self.log
                    .push(format!("Respuesta de {station}: pong en {rtt}."));
            }
        }
    }
}
