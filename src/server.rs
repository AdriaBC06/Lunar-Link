//! Servidor lunar simulado: responde a las peticiones con contenido fijo.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Ping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Pong { station: &'static str },
}

pub const STATION: &str = "Base Tranquilidad";

pub fn handle(request: &Request) -> Response {
    match request {
        Request::Ping => Response::Pong { station: STATION },
    }
}
