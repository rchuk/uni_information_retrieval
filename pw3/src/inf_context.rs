use anyhow::{anyhow, Result, Context};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::document::{Document, DocumentRegistry};
use crate::file::FilePool;
use crate::document::DocumentId;

pub struct InfContext {
    documents: DocumentRegistry,
    files: FilePool
}

impl InfContext {
    pub fn new(base_path: &str) -> Result<Arc<Self>> {
        let mut file_names = get_files(base_path)?;
        let mut files = FilePool::new();
        let mut documents = DocumentRegistry::new();

        for path in file_names.drain(..) {
            let file_id = files.add_file(&path)?;
            documents.add_document(Document::File { path, file_id });
        }

        Ok(Arc::new(InfContext {
            documents,
            files
        }))
    }

    pub fn document_count(&self) -> usize {
        self.documents.document_count()
    }

    pub fn document_ids(&self) -> impl Iterator<Item = DocumentId> + '_ {
        self.documents.document_ids()
    }

    pub fn document(&self, document_id: DocumentId) -> Option<&Document> {
        self.documents.document(document_id)
    }

    pub fn document_data(&self, document_id: DocumentId) -> Result<&str> {
        let document = self.documents.document(document_id)
            .context(anyhow!("Document with id {document_id} doesn't exist"))?;
        match document {
            Document::File { file_id, .. } => {
                let file = self.files.file(*file_id)
                    .context(anyhow!("File with id {file_id} doesn't exist"))?;

                Ok(file.str())
            }
        }
    }
}

fn get_files(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    Ok(std::fs::read_dir(path)?
        .map(|entry| entry.ok())
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect())
}
