//! Contenido premontado del servidor lunar, embebido desde `content.toml`.

use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Mail {
    pub from: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub size_kb: u32,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    pub title: String,
    pub art: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OptionEntry {
    pub label: String,
    pub goto: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Node {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub options: Vec<OptionEntry>,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    #[serde(rename = "mail")]
    pub mails: Vec<Mail>,
    #[serde(rename = "file")]
    pub files: Vec<FileEntry>,
    #[serde(rename = "image")]
    pub images: Vec<Image>,
    #[serde(rename = "node")]
    pub nodes: Vec<Node>,
}

impl Content {
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Busca el nodo con más palabras clave presentes en el texto escrito.
    ///
    /// Se comparan palabras completas, así que «escena» no activa «cena».
    pub fn node_by_text(&self, text: &str) -> Option<&Node> {
        let words: Vec<String> = normalize(text)
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(str::to_string)
            .collect();
        let score = |n: &Node| {
            n.keywords
                .iter()
                .filter(|k| words.contains(&normalize(k)))
                .count()
        };
        // En caso de empate gana el nodo que aparece antes en el contenido.
        self.nodes
            .iter()
            .map(|n| (score(n), n))
            .filter(|(s, _)| *s > 0)
            .fold(None, |best: Option<(usize, &Node)>, cur| match best {
                Some(b) if b.0 >= cur.0 => Some(b),
                _ => Some(cur),
            })
            .map(|(_, n)| n)
    }
}

/// Minúsculas, sin tildes y con la ñ como n, para comparar palabras clave.
fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            'ñ' => 'n',
            c => c,
        })
        .collect()
}

pub fn content() -> &'static Content {
    static CONTENT: OnceLock<Content> = OnceLock::new();
    CONTENT
        .get_or_init(|| toml::from_str(include_str!("content.toml")).expect("content.toml válido"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_loads_and_is_not_empty() {
        let c = content();
        assert!(!c.mails.is_empty() && !c.files.is_empty());
        assert!(!c.images.is_empty() && !c.nodes.is_empty());
    }

    #[test]
    fn dialogue_links_are_valid() {
        let c = content();
        assert!(c.node("inicio").is_some() && c.node("no_entiendo").is_some());
        for node in &c.nodes {
            for option in &node.options {
                assert!(
                    c.node(&option.goto).is_some(),
                    "{} -> {}",
                    node.id,
                    option.goto
                );
            }
        }
    }

    #[test]
    fn keywords_ignore_case_and_accents() {
        let c = content();
        assert_eq!(
            c.node_by_text("¿Dónde ESTOY?").map(|n| n.id.as_str()),
            Some("donde")
        );
        assert!(c.node_by_text("xyzzy").is_none());
    }

    #[test]
    fn keywords_match_whole_words_and_the_best_node() {
        let c = content();
        let id = |s: &str| c.node_by_text(s).map(|n| n.id.as_str());
        assert_eq!(id("¿Cuánto tarda la señal?"), Some("senal"));
        assert_eq!(id("una escena"), None, "«cena» no está dentro de «escena»");
        assert_eq!(id("¿cuánto tarda la luz a Marte?"), Some("senal"));
        assert_eq!(id("háblame de marte, el planeta rojo"), Some("marte"));
    }
}
