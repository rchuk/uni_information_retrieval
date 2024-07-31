use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::file::FileId;

#[derive(Ord, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct DocumentId(pub usize);

impl DocumentId {
    pub fn id(&self) -> usize {
        self.0
    }
}

impl Display for DocumentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Document({})", self.0)
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct DocumentRegistry {
    documents: Vec<Document>
}

impl DocumentRegistry {
    pub fn new() -> Self {
        DocumentRegistry {
            documents: Vec::new()
        }
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn document(&self, document_id: DocumentId) -> Option<&Document> {
        self.documents.get(document_id.0)
    }

    pub fn document_ids(&self) -> impl Iterator<Item = DocumentId> + '_ {
        (0..self.documents.len())
            .map(|id| DocumentId(id))
    }

    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        self.documents.iter()
    }

    pub fn add_document(&mut self, document: Document) -> DocumentId {
        let id = self.documents.len();
        self.documents.push(document);

        DocumentId(id)
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum Document {
    File { path: PathBuf, file_id: FileId }
}

impl Document {
    pub fn name(&self) -> String {
        match self {
            Document::File { path, .. } => path.to_string_lossy().to_string()
        }
    }
}
