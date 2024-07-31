use anyhow::Result;
use std::sync::Arc;
use crate::document::DocumentRegistry;
use crate::term_index::{InvertedIndex, TermMatrix};
use crate::lexer::{Lexer, LexerStats};
use crate::position::DocumentId;

pub fn add_file_to_index(document_registry: Arc<DocumentRegistry>, document_id: DocumentId) -> Result<Option<(InvertedIndex, TermMatrix, LexerStats)>> {
    let document = document_registry.get_document(document_id)?;

    let mut inverted_index = InvertedIndex::new();
    let mut matrix_index = TermMatrix::new();
    let lexer = Lexer::new(document.clone());
    let stats = lexer.lex(&mut inverted_index);
    let lexer1 = Lexer::new(document.clone());
    lexer1.lex(&mut matrix_index);

    Ok(Some((inverted_index, matrix_index, stats)))
}
