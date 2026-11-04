//! Servidor lunar simulado: responde a las peticiones con contenido fijo.

use std::f64::consts::TAU;
use std::time::Duration;

use crate::content::{OptionEntry, content};
use crate::link::Target;

pub const STATION: &str = "Base Tranquilidad";

/// Duración compacta del día lunar en tiempo de simulación (el real dura ~29,5 días).
const LUNAR_DAY_S: f64 = 90.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorInput {
    /// Opción de menú elegida (id del nodo).
    Goto(String),
    /// Texto escrito por el usuario, que se compara con palabras clave.
    Say(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Ping,
    /// Traza la ruta; los saltos dependen del destino del enlace.
    Traceroute(Target),
    ListMail,
    ReadMail(usize),
    ListFiles,
    Download(usize),
    Telemetry,
    Operator(OperatorInput),
    Gallery(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailHeader {
    pub from: String,
    pub subject: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub name: String,
    pub size_kb: u32,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hop {
    pub name: &'static str,
    /// Fracción del trayecto de ida recorrida al llegar a este salto (0 a 1).
    pub fraction: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub temperature_c: f64,
    pub solar_w: f64,
    pub battery_pct: f64,
    pub dust_ugm3: f64,
    pub radiation_usv_h: f64,
    pub sun_up: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Pong {
        station: &'static str,
    },
    Route(Vec<Hop>),
    MailList(Vec<MailHeader>),
    Mail {
        from: String,
        subject: String,
        body: String,
    },
    Files(Vec<FileInfo>),
    FileReceived {
        name: String,
        size_kb: u32,
    },
    Telemetry(Reading),
    Operator {
        text: String,
        options: Vec<OptionEntry>,
    },
    Image {
        index: usize,
        total: usize,
        title: String,
        art: String,
    },
    NotFound,
}

/// Tamaño en bytes del archivo `index`, para calcular el tiempo de descarga.
pub fn file_size_bytes(index: usize) -> Option<u64> {
    content()
        .files
        .get(index)
        .map(|f| u64::from(f.size_kb) * 1024)
}

/// Atiende una petición. `mission` es el tiempo transcurrido desde que arrancó la simulación.
pub fn handle(request: &Request, mission: Duration) -> Response {
    let c = content();
    match request {
        Request::Ping => Response::Pong { station: STATION },
        Request::Traceroute(target) => Response::Route(route(*target)),
        Request::ListMail => Response::MailList(
            c.mails
                .iter()
                .map(|m| MailHeader {
                    from: m.from.clone(),
                    subject: m.subject.clone(),
                })
                .collect(),
        ),
        Request::ReadMail(i) => match c.mails.get(*i) {
            Some(m) => Response::Mail {
                from: m.from.clone(),
                subject: m.subject.clone(),
                body: m.body.clone(),
            },
            None => Response::NotFound,
        },
        Request::ListFiles => Response::Files(
            c.files
                .iter()
                .map(|f| FileInfo {
                    name: f.name.clone(),
                    size_kb: f.size_kb,
                    description: f.description.clone(),
                })
                .collect(),
        ),
        Request::Download(i) => match c.files.get(*i) {
            Some(f) => Response::FileReceived {
                name: f.name.clone(),
                size_kb: f.size_kb,
            },
            None => Response::NotFound,
        },
        Request::Telemetry => Response::Telemetry(reading(mission)),
        Request::Operator(input) => {
            let node = match input {
                OperatorInput::Goto(id) => c.node(id),
                OperatorInput::Say(text) => c.node_by_text(text),
            }
            .or_else(|| c.node("no_entiendo"));
            match node {
                Some(n) => Response::Operator {
                    text: n.text.clone(),
                    options: n.options.clone(),
                },
                None => Response::NotFound,
            }
        }
        Request::Gallery(i) => {
            let total = c.images.len();
            match c.images.get(*i) {
                Some(img) => Response::Image {
                    index: *i,
                    total,
                    title: img.title.clone(),
                    art: img.art.trim_matches('\n').to_string(),
                },
                None => Response::NotFound,
            }
        }
    }
}

/// Saltos del traceroute. `fraction` es la parte del trayecto de ida ya recorrida.
fn route(target: Target) -> Vec<Hop> {
    let hops: &[(&'static str, f64)] = match target {
        Target::Luna => &[
            ("terminal (tú)", 0.0),
            ("Goldstone (Tierra)", 0.002),
            ("relé en órbita lunar", 0.97),
            ("antena de la base", 0.999),
            ("servidor de la base", 1.0),
        ],
        Target::Marte => &[
            ("terminal (tú)", 0.0),
            ("DSN Robledo (Madrid)", 0.000_01),
            ("relé en órbita marciana", 0.999_99),
            ("antena de la base", 0.999_999),
            ("servidor de la base", 1.0),
        ],
    };
    hops.iter()
        .map(|&(name, fraction)| Hop { name, fraction })
        .collect()
}

/// Telemetría determinista según el ciclo día/noche lunar.
fn reading(mission: Duration) -> Reading {
    let t = mission.as_secs_f64();
    let sun = (TAU * t / LUNAR_DAY_S).sin();
    Reading {
        temperature_c: -20.0 + 140.0 * sun,
        solar_w: (480.0 * sun).max(0.0),
        battery_pct: 62.0 + 33.0 * (TAU * t / LUNAR_DAY_S - 0.9).sin(),
        dust_ugm3: 14.0 + 6.0 * (t * 0.37).sin() + 3.0 * (t * 1.9).sin(),
        radiation_usv_h: 0.31 + 0.08 * (t * 0.11).sin(),
        sun_up: sun > 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO: Duration = Duration::ZERO;

    #[test]
    fn every_content_item_is_reachable() {
        let c = content();
        for i in 0..c.mails.len() {
            assert!(matches!(
                handle(&Request::ReadMail(i), ZERO),
                Response::Mail { .. }
            ));
        }
        for i in 0..c.files.len() {
            assert!(matches!(
                handle(&Request::Download(i), ZERO),
                Response::FileReceived { .. }
            ));
            assert!(file_size_bytes(i).is_some());
        }
        for i in 0..c.images.len() {
            assert!(matches!(
                handle(&Request::Gallery(i), ZERO),
                Response::Image { .. }
            ));
        }
        assert_eq!(handle(&Request::ReadMail(999), ZERO), Response::NotFound);
    }

    #[test]
    fn route_depends_on_target_and_ends_at_the_server() {
        for target in [Target::Luna, Target::Marte] {
            let Response::Route(hops) = handle(&Request::Traceroute(target), ZERO) else {
                panic!()
            };
            assert!(hops.windows(2).all(|w| w[0].fraction < w[1].fraction));
            assert_eq!(hops.last().unwrap().fraction, 1.0);
        }
        let Response::Route(mars) = handle(&Request::Traceroute(Target::Marte), ZERO) else {
            panic!()
        };
        assert!(mars.iter().any(|h| h.name.contains("marciana")));
    }

    #[test]
    fn operator_understands_keywords_and_falls_back() {
        let say = |s: &str| handle(&Request::Operator(OperatorInput::Say(s.into())), ZERO);
        let Response::Operator { text, .. } = say("¿hay hielo?") else {
            panic!()
        };
        assert!(text.contains("hielo"));
        let Response::Operator { text, .. } = say("asdfgh") else {
            panic!()
        };
        assert!(text.contains("No he entendido"));
    }

    #[test]
    fn telemetry_follows_the_lunar_day() {
        let day = reading(Duration::from_secs_f64(LUNAR_DAY_S * 0.25));
        let night = reading(Duration::from_secs_f64(LUNAR_DAY_S * 0.75));
        assert!(day.sun_up && !night.sun_up);
        assert!(day.temperature_c > 0.0 && night.temperature_c < -100.0);
        assert!(day.solar_w > 0.0 && night.solar_w == 0.0);
        assert!((0.0..=100.0).contains(&day.battery_pct));
    }
}
