use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const STANDARD_KEYS: &[&str] = &[
    "id",
    "type",
    "title",
    "description",
    "resource",
    "tags",
    "timestamp",
];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Frontmatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub doc_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub frontmatter: Frontmatter,
    pub body: String,
    pub md_links: Vec<String>,
}

impl Document {
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", path.display(), e))?;
        Self::from_str(&content, path.to_path_buf())
    }

    pub fn from_str(content: &str, path: PathBuf) -> anyhow::Result<Self> {
        let parsed = fronma::parser::parse::<Frontmatter>(content)
            .map_err(|e| anyhow::anyhow!("Frontmatter error in {}: {:?}", path.display(), e))?;
        let md_links = extract_md_links(parsed.body);
        Ok(Self {
            path,
            frontmatter: parsed.headers,
            body: parsed.body.to_string(),
            md_links,
        })
    }

    pub fn filename(&self) -> &str {
        self.path.file_name().and_then(|n| n.to_str()).unwrap_or("")
    }

    pub fn is_index(&self) -> bool {
        self.filename() == "index.md"
    }

    pub fn is_log(&self) -> bool {
        self.filename() == "log.md"
    }

    pub fn display_title(&self) -> String {
        self.frontmatter
            .title
            .clone()
            .unwrap_or_else(|| self.filename().to_string())
    }
}

/// Returns the YAML text between the opening and closing `---` markers.
pub fn frontmatter_yaml_block(content: &str) -> Option<&str> {
    if !content.starts_with("---") {
        return None;
    }
    let after = &content[3..];
    let end = after.find("\n---")?;
    Some(&after[..end])
}

/// Returns the byte offset just after the closing `---\n` of the YAML frontmatter.
pub fn frontmatter_end(content: &str) -> Option<usize> {
    if !content.starts_with("---") {
        return None;
    }
    let after = &content[3..];
    let pos = after.find("\n---")?;
    let end = 3 + pos + 4;
    if content[end..].starts_with('\n') {
        Some(end + 1)
    } else {
        Some(end)
    }
}

fn extract_md_links(text: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\[[^\]]*\]\(([^)]+\.md(?:#[^)]*)?)\)").unwrap();
    re.captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_FM: &str = "\
---
id: \"abc-123\"
type: \"Concept\"
title: \"Test Doc\"
description: \"A test\"
resource: \"https://example.com\"
tags:
  - rust
  - tui
timestamp: \"2026-01-01T00:00:00Z\"
---

# Test Doc
";

    const NO_OPTIONAL_FM: &str = "\
---
type: \"Concept\"
---

Body only.
";

    const NO_FM: &str = "# Just Markdown\n\nNo frontmatter here.\n";

    #[test]
    fn parse_full_frontmatter() {
        let doc = Document::from_str(FULL_FM, PathBuf::from("test.md")).unwrap();
        assert_eq!(doc.frontmatter.id.as_deref(), Some("abc-123"));
        assert_eq!(doc.frontmatter.doc_type, "Concept");
        assert_eq!(doc.frontmatter.title.as_deref(), Some("Test Doc"));
        assert_eq!(doc.frontmatter.description.as_deref(), Some("A test"));
        assert_eq!(
            doc.frontmatter.tags.as_ref().unwrap(),
            &["rust".to_string(), "tui".to_string()]
        );
        assert_eq!(
            doc.frontmatter.timestamp.as_deref(),
            Some("2026-01-01T00:00:00Z")
        );
    }

    #[test]
    fn parse_optional_fields_absent() {
        let doc = Document::from_str(NO_OPTIONAL_FM, PathBuf::from("test.md")).unwrap();
        assert_eq!(doc.frontmatter.doc_type, "Concept");
        assert!(doc.frontmatter.id.is_none());
        assert!(doc.frontmatter.title.is_none());
        assert!(doc.frontmatter.tags.is_none());
        assert!(doc.frontmatter.timestamp.is_none());
    }

    #[test]
    fn parse_no_frontmatter_returns_err() {
        let result = Document::from_str(NO_FM, PathBuf::from("test.md"));
        assert!(result.is_err());
    }

    #[test]
    fn extract_links_finds_md_links() {
        let text = "See [Ownership](ownership.md) and [Borrow](borrow.md).";
        let links = extract_md_links(text);
        assert_eq!(links, vec!["ownership.md", "borrow.md"]);
    }

    #[test]
    fn extract_links_ignores_http() {
        let text = "Visit [site](https://example.com) or read [doc](local.md).";
        let links = extract_md_links(text);
        assert_eq!(links, vec!["local.md"]);
    }

    #[test]
    fn extract_links_with_anchor() {
        let text = "See [section](other.md#overview).";
        let links = extract_md_links(text);
        assert_eq!(links, vec!["other.md#overview"]);
    }

    #[test]
    fn is_index_true_for_index_md() {
        let doc = Document::from_str(NO_OPTIONAL_FM, PathBuf::from("index.md")).unwrap();
        assert!(doc.is_index());
        assert!(!doc.is_log());
    }

    #[test]
    fn is_log_true_for_log_md() {
        let doc = Document::from_str(NO_OPTIONAL_FM, PathBuf::from("log.md")).unwrap();
        assert!(doc.is_log());
        assert!(!doc.is_index());
    }

    #[test]
    fn frontmatter_end_basic() {
        // frontmatter_end skips exactly one \n after the closing ---
        let content = "---\ntype: Concept\n---\nBody";
        let offset = frontmatter_end(content).unwrap();
        assert_eq!(&content[offset..], "Body");
    }

    #[test]
    fn frontmatter_end_no_marker_returns_none() {
        assert!(frontmatter_end("# No frontmatter").is_none());
    }
}
