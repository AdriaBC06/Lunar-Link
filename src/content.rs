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

    /// Busca el nodo cuyas palabras clave aparecen en el texto escrito.
    pub fn node_by_text(&self, text: &str) -> Option<&Node> {
        let text = normalize(text);
        self.nodes
            .iter()
            .find(|n| n.keywords.iter().any(|k| text.contains(&normalize(k))))
    }
}

/// Minúsculas y sin tildes, para comparar palabras clave.
fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
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
}
