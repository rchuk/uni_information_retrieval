use std::path::Path;
use crate::dictionary::Dictionary;
use crate::document::Document;
use crate::lexer::{Lexer, LexerStats};

pub fn add_file_to_dict(path: impl AsRef<Path>) -> anyhow::Result<Option<(Dictionary, LexerStats)>> {
    if let Some(document) = Document::new(path)? {
        let mut dict = Dictionary::new();
        let lexer = Lexer::new(&document)?;
        let stats = lexer.lex_to_dictionary(&mut dict);

        Ok(Some((dict, stats)))
    } else {
        Ok(None)
    }
}
