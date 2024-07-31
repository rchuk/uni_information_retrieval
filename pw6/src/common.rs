use anyhow::Result;
use std::sync::Arc;
use crate::inf_context::InfContext;
use crate::term_index::InvertedIndex;
use crate::lexer::{Lexer, LexerStats};
use crate::document::DocumentId;

pub fn add_file_to_index(document_id: DocumentId, ctx: Arc<InfContext>) -> Result<Option<(InvertedIndex, LexerStats)>> {
    let mut inverted_index = InvertedIndex::new();
    let lexer = Lexer::new(document_id, &ctx)?;
    let stats = lexer.lex(&mut inverted_index);
    inverted_index.shrink_to_fit();

    Ok(Some((inverted_index, stats)))
}
