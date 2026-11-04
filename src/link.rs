//! Modelo del enlace Tierra-destino: retardo variable, cortes de señal y ancho de banda.

use std::f64::consts::TAU;
use std::time::Duration;

use clap::ValueEnum;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{Instant, sleep};

use crate::server::{self, Request, Response};

/// Velocidad de la luz en km/s.
const LIGHT_SPEED_KM_S: f64 = 299_792.458;
/// Ancho de banda por defecto del enlace, en bytes por segundo.
pub const DEFAULT_BANDWIDTH: u64 = 24 * 1024;
/// Tiempo de renegociación del enlace tras recuperar la señal.
const RECONNECT: Duration = Duration::from_millis(300);
/// Intervalo de envío de datos durante una descarga.
const CHUNK_INTERVAL: Duration = Duration::from_millis(100);

/// Período de la órbita del relé en tiempo de simulación (la real dura ~27 días).
const ORBIT_S: f64 = 90.0;
/// Segundos de cada ciclo orbital en los que el destino queda oculto.
const HIDDEN_S: f64 = 10.0;
/// Duración de la ventana en la que puede producirse un corte aleatorio.
const SLOT_S: f64 = 20.0;
/// Segundos iniciales sin cortes, para que arranque tranquilo.
const GRACE_S: f64 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Target {
    Luna,
    Marte,
}

impl Target {
    pub fn label(self) -> &'static str {
        match self {
            Target::Luna => "Luna",
            Target::Marte => "Marte",
        }
    }

    /// Nombre con artículo, para frases ("viajando a la Luna").
    pub fn destination(self) -> &'static str {
        match self {
            Target::Luna => "la Luna",
            Target::Marte => "Marte",
        }
    }

    /// Distancia media, amplitud y período (s) de la oscilación de la distancia.
    ///
    /// La Luna oscila entre ~363.000 y ~405.000 km; Marte entre 3 y 22 minutos luz.
    /// Los períodos están comprimidos para que la variación se note durante una sesión.
    fn orbit(self) -> (f64, f64, f64) {
        match self {
            Target::Luna => (384_400.0, 21_100.0, 120.0),
            Target::Marte => {
                let (min, max) = (
                    3.0 * 60.0 * LIGHT_SPEED_KM_S,
                    22.0 * 60.0 * LIGHT_SPEED_KM_S,
                );
                ((min + max) / 2.0, (max - min) / 2.0, 600.0)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutageKind {
    /// Pérdida de comunicación aleatoria.
    Random,
    /// El destino queda detrás de su cuerpo (cara oculta de la Luna) o del Sol (Marte).
    Hidden,
    SolarStorm,
}

impl OutageKind {
    pub fn label(self, target: Target) -> &'static str {
        match (self, target) {
            (OutageKind::Random, _) => "pérdida de comunicación",
            (OutageKind::Hidden, Target::Luna) => "cara oculta de la Luna",
            (OutageKind::Hidden, Target::Marte) => "conjunción solar",
            (OutageKind::SolarStorm, _) => "tormenta solar",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outage {
    pub kind: OutageKind,
    /// Instante de la sesión en el que termina el corte.
    pub ends_at: Duration,
}

/// Evento que el enlace entrega al cliente.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkEvent {
    /// Avance de una descarga en curso.
    Progress {
        id: u64,
        done: u64,
        total: u64,
    },
    Response {
        id: u64,
        response: Response,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Link {
    pub target: Target,
    /// Retardo de ida fijo que sustituye al del destino (útil para pruebas).
    pub fixed_one_way: Option<Duration>,
    pub outages: bool,
    pub bandwidth: u64,
    /// Factor de aceleración del tiempo de viaje (retardos y descargas); los cortes no cambian.
    pub speed: f64,
    seed: u64,
    start: Instant,
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

impl Link {
    pub fn new(target: Target, fixed_one_way: Option<Duration>, outages: bool, seed: u64) -> Self {
        Self {
            target,
            fixed_one_way,
            outages,
            bandwidth: DEFAULT_BANDWIDTH,
            speed: 1.0,
            seed,
            start: Instant::now(),
        }
    }

    /// Duración real que tarda en transcurrir un intervalo de tiempo simulado.
    fn real(&self, simulated: Duration) -> Duration {
        simulated.div_f64(self.speed)
    }

    /// Tiempo real transcurrido desde que arrancó el enlace.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn distance_km(&self, t: Duration) -> f64 {
        match self.fixed_one_way {
            Some(d) => d.as_secs_f64() * LIGHT_SPEED_KM_S,
            None => {
                let (mean, amp, period) = self.target.orbit();
                mean + amp * (TAU * t.as_secs_f64() / period).sin()
            }
        }
    }

    pub fn one_way_at(&self, t: Duration) -> Duration {
        Duration::from_secs_f64(self.distance_km(t) / LIGHT_SPEED_KM_S)
    }

    pub fn one_way(&self) -> Duration {
        self.one_way_at(self.elapsed())
    }

    pub fn round_trip(&self) -> Duration {
        self.one_way() * 2
    }

    /// Corte activo en el instante `t`, si lo hay. Es determinista según la semilla.
    pub fn status_at(&self, t: Duration) -> Option<Outage> {
        if !self.outages {
            return None;
        }
        let t = t.as_secs_f64();
        if t < GRACE_S {
            return None;
        }

        let phase = t % ORBIT_S;
        if phase >= ORBIT_S - HIDDEN_S {
            let ends_at = Duration::from_secs_f64(t - phase + ORBIT_S);
            return Some(Outage {
                kind: OutageKind::Hidden,
                ends_at,
            });
        }

        let slot = (t / SLOT_S) as u64;
        let h = splitmix64(self.seed ^ slot);
        let (kind, len) = match h % 100 {
            0..=14 => (OutageKind::Random, 2.0 + ((h >> 8) % 4) as f64),
            15..=21 => (OutageKind::SolarStorm, 6.0 + ((h >> 8) % 5) as f64),
            _ => return None,
        };
        let room_ms = ((SLOT_S - len) * 1000.0) as u64;
        let start = slot as f64 * SLOT_S + ((h >> 16) % room_ms) as f64 / 1000.0;
        (t >= start && t < start + len).then(|| Outage {
            kind,
            ends_at: Duration::from_secs_f64(start + len),
        })
    }

    /// Calidad de la señal (0-100); 0 durante un corte.
    pub fn signal_quality(&self, t: Duration) -> u8 {
        if self.status_at(t).is_some() {
            return 0;
        }
        let jitter = splitmix64(self.seed ^ t.as_secs() ^ 0xA5A5) % 26;
        (75 + jitter) as u8
    }

    /// Espera a que vuelva la señal, reintentando tras cada corte.
    async fn wait_for_signal(&self) {
        while let Some(outage) = self.status_at(self.elapsed()) {
            sleep(outage.ends_at.saturating_sub(self.elapsed()) + RECONNECT).await;
        }
    }

    /// Envía una petición: retardo de ida, servidor, (descarga) y retardo de vuelta.
    pub async fn transmit(&self, id: u64, request: Request, tx: UnboundedSender<LinkEvent>) {
        self.wait_for_signal().await;
        sleep(self.real(self.one_way())).await;

        if let Request::Download(index) = &request
            && let Some(total) = server::file_size_bytes(*index)
        {
            self.stream(id, total, &tx).await;
        }

        let response = server::handle(&request, self.elapsed());
        self.wait_for_signal().await;
        sleep(self.real(self.one_way())).await;
        let _ = tx.send(LinkEvent::Response { id, response });
    }

    /// Transfiere `total` bytes limitados por el ancho de banda; los cortes la pausan.
    async fn stream(&self, id: u64, total: u64, tx: &UnboundedSender<LinkEvent>) {
        let chunk =
            (self.bandwidth as f64 * self.speed * CHUNK_INTERVAL.as_secs_f64()).max(1.0) as u64;
        let mut done = 0;
        while done < total {
            self.wait_for_signal().await;
            sleep(CHUNK_INTERVAL).await;
            done = (done + chunk).min(total);
            let _ = tx.send(LinkEvent::Progress { id, done, total });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::time::advance;

    fn link(target: Target, fixed: Option<Duration>, outages: bool) -> Link {
        Link::new(target, fixed, outages, 42)
    }

    async fn round_trip(link: &Link, request: Request) -> (Duration, Vec<LinkEvent>) {
        let (tx, mut rx) = unbounded_channel();
        let start = Instant::now();
        link.transmit(0, request, tx).await;
        let elapsed = start.elapsed();
        let mut events = Vec::new();
        while let Ok(e) = rx.try_recv() {
            events.push(e);
        }
        (elapsed, events)
    }

    #[tokio::test(start_paused = true)]
    async fn moon_ping_takes_about_2_6_seconds() {
        let (elapsed, events) = round_trip(&link(Target::Luna, None, false), Request::Ping).await;
        let secs = elapsed.as_secs_f64();
        assert!((2.5..2.7).contains(&secs), "tardó {secs} s");
        assert!(matches!(
            events.last(),
            Some(LinkEvent::Response {
                response: Response::Pong { .. },
                ..
            })
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn fixed_delay_is_applied_both_ways() {
        let l = link(Target::Luna, Some(Duration::from_secs(5)), false);
        let (elapsed, _) = round_trip(&l, Request::Ping).await;
        assert_eq!(elapsed, Duration::from_secs(10));
    }

    #[test]
    fn moon_delay_stays_within_real_range() {
        let l = link(Target::Luna, None, false);
        for s in 0..600 {
            let km = l.distance_km(Duration::from_secs(s));
            assert!((363_000.0..=405_600.0).contains(&km), "{km} km en t={s}");
        }
        // Ida y vuelta entre ~2,4 y ~2,7 s.
        let rtts: Vec<f64> = (0..600)
            .map(|s| l.one_way_at(Duration::from_secs(s)).as_secs_f64() * 2.0)
            .collect();
        assert!(rtts.iter().all(|r| (2.4..2.72).contains(r)));
        assert!(
            rtts.iter().cloned().fold(f64::MAX, f64::min) < 2.5,
            "el retardo debe variar"
        );
    }

    #[test]
    fn mars_delay_is_between_3_and_22_minutes() {
        let l = link(Target::Marte, None, false);
        for s in (0..7200).step_by(30) {
            let m = l.one_way_at(Duration::from_secs(s)).as_secs_f64() / 60.0;
            assert!((2.99..=22.01).contains(&m), "{m} min en t={s}");
        }
    }

    #[test]
    fn outages_are_deterministic_and_follow_the_orbit() {
        let l = link(Target::Luna, None, true);
        // Sin cortes durante el periodo de gracia.
        assert!((0..8).all(|s| l.status_at(Duration::from_secs(s)).is_none()));
        // Cara oculta al final de cada órbita.
        let hidden = l.status_at(Duration::from_secs(85)).unwrap();
        assert_eq!(hidden.kind, OutageKind::Hidden);
        assert_eq!(hidden.ends_at, Duration::from_secs(90));
        // Hay cortes de otros tipos y siempre terminan en el futuro.
        let mut kinds = std::collections::HashSet::new();
        for ms in (0..1_800_000).step_by(250) {
            let t = Duration::from_millis(ms);
            if let Some(o) = l.status_at(t) {
                assert!(o.ends_at > t);
                kinds.insert(format!("{:?}", o.kind));
            }
        }
        assert_eq!(kinds.len(), 3, "{kinds:?}");
        // Sin la opción activada nunca hay cortes.
        assert!(
            link(Target::Luna, None, false)
                .status_at(Duration::from_secs(85))
                .is_none()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn request_waits_out_a_blackout_and_retries() {
        let l = link(Target::Luna, Some(Duration::from_secs(1)), true);
        advance(Duration::from_secs(82)).await; // cara oculta hasta t=90
        let (elapsed, events) = round_trip(&l, Request::Ping).await;
        // 8 s de corte + reconexión + 2 s de viaje.
        assert!(elapsed >= Duration::from_millis(10_300), "{elapsed:?}");
        assert!(matches!(
            events.last(),
            Some(LinkEvent::Response {
                response: Response::Pong { .. },
                ..
            })
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn download_time_scales_with_size_and_bandwidth() {
        let mut l = link(Target::Luna, Some(Duration::from_secs(1)), false);
        l.bandwidth = 10 * 1024;
        let size = server::file_size_bytes(0).unwrap() as f64;
        let (elapsed, events) = round_trip(&l, Request::Download(0)).await;
        let expected = size / l.bandwidth as f64 + 2.0;
        assert!(
            (elapsed.as_secs_f64() - expected).abs() < 0.3,
            "{elapsed:?} vs {expected}"
        );
        let last = events.iter().rev().find_map(|e| match e {
            LinkEvent::Progress { done, total, .. } => Some((*done, *total)),
            _ => None,
        });
        assert_eq!(last, Some((size as u64, size as u64)));
        assert!(matches!(
            events.last(),
            Some(LinkEvent::Response {
                response: Response::FileReceived { .. },
                ..
            })
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn speed_compresses_travel_time_but_not_signal_schedule() {
        let mut l = link(Target::Marte, None, false);
        l.speed = 100.0;
        // Ida y vuelta simulada de ~25 min (a Marte) pasa en ~15 s reales.
        let (elapsed, _) = round_trip(&l, Request::Ping).await;
        let secs = elapsed.as_secs_f64();
        assert!((14.0..16.0).contains(&secs), "tardó {secs} s reales");

        // Los cortes dependen del tiempo real: igual con cualquier velocidad.
        let slow = link(Target::Marte, None, true);
        let fast = Link {
            speed: 100.0,
            ..slow
        };
        let t = Duration::from_secs(85);
        assert_eq!(slow.status_at(t), fast.status_at(t));
    }
}
