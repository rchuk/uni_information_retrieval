use std::str::CharIndices;
use std::sync::Arc;
use crate::document::Document;
use crate::position::TermDocumentPosition;
use crate::term_index::TermIndex;

pub struct Lexer {
    document: Arc<Document>,
    iter: CharIndices<'static>
}

impl Lexer {
    pub fn new(document: Arc<Document>) -> Self {
        let iter = unsafe { std::mem::transmute(document.str().char_indices()) };

        Lexer {
            document,
            iter
        }
    }

    pub fn lex(mut self, term_index: &mut dyn TermIndex) -> LexerStats {
        let mut pos = 0;
        let mut word = String::new();
        let mut stats = LexerStats::default();
        stats.lines += 1;

        while let Some((cursor, ch)) = self.iter.next() {
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
                let mut new_word = String::new();
                std::mem::swap(&mut word, &mut new_word);

                new_word.shrink_to_fit();
                term_index.add_term(new_word, self.document.id(), TermDocumentPosition::new(pos));
                pos = cursor;
            }
        }

        if !word.is_empty() {
            word.shrink_to_fit();
            term_index.add_term(word, self.document.id(), TermDocumentPosition::new(pos));
        }

        stats
    }
}

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
