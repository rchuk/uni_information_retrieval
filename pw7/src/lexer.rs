use anyhow::Result;
use std::str::Chars;
use crate::document::DocumentId;
use crate::inf_context::InfContext;
use crate::segment::{SegmentKind, TermPosition};
use crate::term_index::TermIndex;

pub struct Lexer<'a> {
    document_id: DocumentId,
    iter: Chars<'a>
}

impl<'a> Lexer<'a> {
    pub fn new(document_id: DocumentId, data: &'a str, ctx: &'a InfContext) -> Result<Self> {
        let iter = data.chars();

        Ok(Lexer {
            document_id,
            iter
        })
    }

    pub fn lex(mut self, term_index: &mut dyn TermIndex, segment_kind: SegmentKind) -> LexerStats {
        let mut word = String::new();
        let mut stats = LexerStats::default();
        stats.lines += 1;

        while let Some(ch) = self.iter.next() {
            stats.characters_read += 1;
            if ch.is_alphabetic() || (ch.eq(&'\'') && !word.is_empty()) {
                ch.to_lowercase().for_each(|ch| word.push(ch));

                continue;
            }

            stats.characters_ignored += 1;
            if ch == '\n' {
                stats.lines += 1;
            }
            if !word.is_empty() {
                Self::add_term(&mut word, TermPosition { document: self.document_id, segment_kind }, term_index);
            }
        }

        if !word.is_empty() {
            Self::add_term(&mut word, TermPosition { document: self.document_id, segment_kind }, term_index);
        }

        stats
    }

    fn add_term(word: &mut String, term_position: TermPosition, term_index: &mut dyn TermIndex) {
        let mut new_word = String::new();
        std::mem::swap(word, &mut new_word);

        new_word.shrink_to_fit();
        term_index.add_term(new_word, term_position);
    }
}

#[derive(Debug)]
pub struct LexerStats {
    pub characters_read: usize,
    pub characters_ignored: usize,
    pub lines: usize
}

impl LexerStats {
    pub fn merge(&mut self, other: LexerStats) {
        self.characters_read += other.characters_read;
        self.characters_ignored += other.characters_ignored;
        self.lines += other.lines;
    }
}

impl Default for LexerStats {
    fn default() -> Self {
        LexerStats {
            characters_read: 0,
            characters_ignored: 0,
            lines: 0
        }
    }
}
