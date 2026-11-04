//! Modelo del enlace Tierra-destino: retardo de ida y vuelta.

use std::time::Duration;

use clap::ValueEnum;
use tokio::time::sleep;

use crate::server::{self, Request, Response};

/// Velocidad de la luz en km/s.
const LIGHT_SPEED_KM_S: f64 = 299_792.458;
/// Distancia media Tierra-Luna en km.
const MOON_MEAN_KM: f64 = 384_400.0;
/// Retardo medio de ida a Marte en segundos (entre 3 y 22 minutos).
const MARS_MEAN_ONE_WAY_S: f64 = 12.5 * 60.0;

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

    pub fn one_way_delay(self) -> Duration {
        match self {
            Target::Luna => Duration::from_secs_f64(MOON_MEAN_KM / LIGHT_SPEED_KM_S),
            Target::Marte => Duration::from_secs_f64(MARS_MEAN_ONE_WAY_S),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Link {
    pub target: Target,
    /// Retardo de ida fijo que sustituye al del destino (útil para pruebas).
    pub fixed_one_way: Option<Duration>,
    pub outages: bool,
}

impl Link {
    pub fn one_way(&self) -> Duration {
        self.fixed_one_way
            .unwrap_or_else(|| self.target.one_way_delay())
    }

    pub fn round_trip(&self) -> Duration {
        self.one_way() * 2
    }

    /// Envía una petición: retardo de ida, servidor, retardo de vuelta.
    pub async fn send(&self, request: Request) -> Response {
        sleep(self.one_way()).await;
        let response = server::handle(&request);
        sleep(self.one_way()).await;
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    fn link(target: Target, fixed: Option<Duration>) -> Link {
        Link {
            target,
            fixed_one_way: fixed,
            outages: false,
        }
    }

    #[tokio::test(start_paused = true)]
    async fn moon_ping_takes_about_2_6_seconds() {
        let start = Instant::now();
        let response = link(Target::Luna, None).send(Request::Ping).await;
        let elapsed = start.elapsed().as_secs_f64();
        assert!((2.5..2.7).contains(&elapsed), "tardó {elapsed} s");
        assert!(matches!(response, Response::Pong { .. }));
    }

    #[tokio::test(start_paused = true)]
    async fn fixed_delay_is_applied_both_ways() {
        let start = Instant::now();
        link(Target::Luna, Some(Duration::from_secs(5)))
            .send(Request::Ping)
            .await;
        assert_eq!(start.elapsed(), Duration::from_secs(10));
    }

    #[test]
    fn mars_delay_is_between_3_and_22_minutes() {
        let s = Target::Marte.one_way_delay().as_secs_f64();
        assert!((180.0..=1320.0).contains(&s));
    }
}
