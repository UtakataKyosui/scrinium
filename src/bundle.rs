use crate::document::Document;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct Bundle {
    pub root: PathBuf,
    pub documents: Vec<Document>,
}

impl Bundle {
    pub fn load(dir: &Path) -> Result<Self> {
        let mut documents = Vec::new();
        for entry in crate::markdown_files(dir) {
            match Document::from_path(entry.path()) {
                Ok(doc) => documents.push(doc),
                Err(e) => eprintln!("Warning: {e}"),
            }
        }
        Ok(Self {
            root: dir.to_path_buf(),
            documents,
        })
    }

    pub fn concept_docs(&self) -> impl Iterator<Item = &Document> {
        self.documents
            .iter()
            .filter(|d| !d.is_index() && !d.is_log())
    }
}
