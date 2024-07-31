use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};
use std::sync::Arc;
use serde::Serialize;
use crate::file::File;
use crate::position::DocumentId;

#[derive(Serialize)]
#[derive(Debug)]
pub struct DocumentRegistry {
    documents: Vec<Arc<Document>>
}

impl DocumentRegistry {
    pub fn new(base_path: &str) -> Result<Arc<Self>> {
        let file_names = get_files(base_path)?;
        let documents = file_names.iter()
            .cloned()
            .map(File::new)
            .flatten()
            .flatten()
            .enumerate()
            .map(|(id, file)| Document::file(DocumentId(id), file))
            .map(Arc::new)
            .collect();

        Ok(Arc::new(DocumentRegistry { documents }))
    }

    pub fn documents_count(&self) -> usize {
        self.documents.len()
    }

    pub fn get_document(&self, document_id: DocumentId) -> Result<Arc<Document>> {
        self.documents.get(document_id.0)
            .cloned()
            .ok_or(anyhow!("Document with id {} doesn't exist", document_id.0))
    }
}

#[derive(Serialize)]
#[derive(Debug)]
pub struct Document {
    id: DocumentId,
    kind: DocumentKind
}

impl Document {
    pub fn file(id: DocumentId, file: File) -> Self {
        Self::new(id, DocumentKind::File(file))
    }

    pub fn new(id: DocumentId, kind: DocumentKind) -> Self {
        Document { id, kind }
    }

    pub fn id(&self) -> DocumentId {
        self.id
    }

    pub fn kind(&self) -> &DocumentKind {
        &self.kind
    }

    pub fn str(&self) -> &str {
        match &self.kind {
            DocumentKind::File(file) => file.str()
        }
    }

    pub fn name(&self) -> String {
        match &self.kind {
            DocumentKind::File(file) => file.path().to_string_lossy().to_string()
        }
    }
}

#[derive(Serialize)]
#[derive(Debug)]
pub enum DocumentKind {
    File(File)
}

fn get_files(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    Ok(std::fs::read_dir(path)?
        .map(|entry| entry.ok())
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect())
}
