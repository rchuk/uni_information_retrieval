use anyhow::Result;
use std::sync::Arc;
use crate::inf_context::InfContext;
use crate::term_index::InvertedIndex;
use crate::lexer::{Lexer, LexerStats};
use crate::document::DocumentId;
use crate::two_word_index::TwoWordIndex;

pub fn add_file_to_index(document_id: DocumentId, ctx: Arc<InfContext>) -> Result<Option<(InvertedIndex, TwoWordIndex, LexerStats)>> {
    let mut inverted_index = InvertedIndex::new();
    let mut two_word_index = TwoWordIndex::new();
    let lexer = Lexer::new(document_id, &ctx)?;
    let stats = lexer.lex(&mut inverted_index);
    let mut lexer1 = Lexer::new(document_id, &ctx)?;
    lexer1.lex(&mut two_word_index);

    Ok(Some((inverted_index, two_word_index, stats)))
}
