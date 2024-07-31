use std::str::{Chars, Utf8Error};
use crate::dictionary::Dictionary;
use crate::document::Document;

pub struct Lexer<'a> {
    document: &'a Document,
    iter: Chars<'a>
}

impl<'a> Lexer<'a> {
    pub fn new(document: &'a Document) -> Result<Self, Utf8Error> {
        Ok(Lexer {
            document,
            iter: document.to_str()?.chars()
        })
    }

    pub fn lex_to_dictionary(mut self, dict: &mut Dictionary) -> LexerStats {
        let mut word = String::new();
        let mut stats = LexerStats::default();
        stats.lines += 1;

        while let Some(ch) = self.next_ch() {
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
                dict.add_word(new_word);
            }
        }

        if !word.is_empty() {
            word.shrink_to_fit();
            dict.add_word(word);
        }

        stats
    }

    fn next_ch(&mut self) -> Option<char> {
        self.iter.next()
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
