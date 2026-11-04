//! Formato de números y duraciones en español: coma decimal y punto de miles.

use std::time::Duration;

/// Número con `prec` decimales y coma decimal: 2.65 → "2,65".
pub fn dec(x: f64, prec: usize) -> String {
    format!("{x:.prec$}").replace('.', ",")
}

/// Separa los miles con puntos: 384512 → "384.512".
pub fn km(km: f64) -> String {
    let digits = (km.round() as u64).to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}

/// Duración legible: "1,3 s" o "12 min 05 s".
pub fn dur(d: Duration) -> String {
    // Se redondea antes de elegir el formato para no mostrar "60,0 s".
    let tenths = (d.as_secs_f64() * 10.0).round();
    if tenths < 600.0 {
        format!("{} s", dec(tenths / 10.0, 1))
    } else {
        let s = d.as_secs_f64().round() as u64;
        format!("{} min {:02} s", s / 60, s % 60)
    }
}

/// Latencia corta en milisegundos ("2,6 ms"); por encima de 10 s, como duración.
pub fn latency(d: Duration) -> String {
    if d < Duration::from_secs(10) {
        format!("{} ms", dec(d.as_secs_f64() * 1000.0, 1))
    } else {
        dur(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spanish_number_format() {
        assert_eq!(km(384_512.4), "384.512");
        assert_eq!(km(999.0), "999");
        assert_eq!(dec(0.35, 2), "0,35");
        assert_eq!(dur(Duration::from_millis(2_650)), "2,7 s");
        assert_eq!(dur(Duration::from_millis(59_970)), "1 min 00 s");
        assert_eq!(dur(Duration::from_secs(725)), "12 min 05 s");
        assert_eq!(latency(Duration::from_micros(2_600)), "2,6 ms");
        assert_eq!(latency(Duration::from_secs(750)), "12 min 30 s");
    }
}
