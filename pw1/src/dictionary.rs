use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Dictionary {
    #[serde(flatten)]
    words: HashMap<String, usize>
}

impl Dictionary {
    pub fn new() -> Self {
        Dictionary {
            words: HashMap::new()
        }
    }

    pub fn word_counts(&self) -> &HashMap<String, usize> {
        &self.words
    }

    pub fn merge(&mut self, mut other: Dictionary) {
        other.words.drain()
            .for_each(|(word, count)| self.add_word_with_count(word, count));
    }

    pub fn unique_word_count(&self) -> usize {
        self.words.len()
    }

    pub fn total_word_count(&self) -> usize {
        self.words.values().sum()
    }

    pub fn add_word(&mut self, word: String) {
        self.add_word_with_count(word, 1);
    }

    pub fn add_word_with_count(&mut self, word: String, count: usize) {
        self.words.entry(word)
            .and_modify(|curr_count| *curr_count += count)
            .or_insert(count);
    }
}
