use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::BitOrAssign;
use bitvec::prelude::BitVec;
use crate::position::{DocumentId, TermDocumentPosition, TermPositions};

pub trait TermIndex {
    fn add_term(&mut self, term: String, document_id: DocumentId, position: TermDocumentPosition);
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct InvertedIndex {
    #[serde(flatten)]
    index: HashMap<String, TermPositions>
}

impl InvertedIndex {
    pub fn new() -> Self {
        InvertedIndex { index: HashMap::new() }
    }

    pub fn unique_word_count(&self) -> usize {
        self.index.len()
    }

    pub fn total_word_count(&self) -> usize {
        self.index.values()
            .map(TermPositions::positions_count)
            .sum()
    }

    pub fn get_term_documents(&self, term: &str) -> HashSet<DocumentId> {
        self.index.get(term)
            .map(|positions| {
                positions.documents().collect()
            })
            .unwrap_or_else(HashSet::new)
    }

    pub fn get_documents(&self) -> HashSet<DocumentId> {
        self.index.values()
            .flat_map(|positions| positions.documents())
            .collect()
    }

    pub fn merge(&mut self, mut other: Self) {
        other.index.drain()
            .for_each(|(term, positions)| self.merge_term_positions(term, positions));
    }

    fn merge_term_positions(&mut self, term: String, positions: TermPositions) {
        self.index.entry(term)
            .or_insert_with(TermPositions::new)
            .merge(positions);
    }
}

impl TermIndex for InvertedIndex {
    fn add_term(&mut self, term: String, document_id: DocumentId, position: TermDocumentPosition) {
        self.index.entry(term)
            .or_insert_with(TermPositions::new)
            .add_position(document_id, position);
    }
}

#[derive(Debug)]
pub struct TermMatrix {
    terms: HashMap<String, usize>,
    rows: Vec<BitVec>,
    col_count: usize
}

impl TermMatrix {
    pub fn new() -> Self {
        TermMatrix {
            terms: HashMap::new(),
            rows: Vec::new(),
            col_count: 0
        }
    }

    pub fn merge(&mut self, mut other: Self) {
        self.col_count = self.col_count.max(other.col_count);
        self.rows.iter_mut()
            .for_each(|row| row.resize(self.col_count, false));

        other.terms.drain()
            .for_each(|(term, other_row)| {
                let other_row = other.rows.get(other_row).unwrap();
                if let Some(&row) = self.terms.get(&term) {
                    let row = self.rows.get_mut(row).unwrap();
                    row.bitor_assign(other_row);
                } else {
                    let row_count = self.rows.len();
                    self.terms.insert(term, row_count);

                    let mut other_row = other_row.clone();
                    other_row.resize(self.col_count, false);
                    self.rows.push(other_row);
                }
            });
    }

    pub fn get_term_query(&self, term: &str) -> BitVec {
        self.terms.get(term)
            .map(|&row| {
                self.rows.get(row).cloned().unwrap()
            })
            .unwrap_or_else(|| {
                let mut query = BitVec::new();
                query.resize(self.col_count, false);

                query
            })
    }

    pub fn get_term_documents(&self, query: &BitVec) -> HashSet<DocumentId> {
        query.iter_ones()
            .map(|i| DocumentId(i))
            .collect()
    }
}

impl TermIndex for TermMatrix {
    fn add_term(&mut self, term: String, document_id: DocumentId, _position: TermDocumentPosition) {
        let col = document_id.0;

        if col >= self.col_count {
            self.col_count = col + 1;

            self.rows.iter_mut()
                .for_each(|row| row.resize(col + 1, false));
        }

        let row = if let Some(&row) = self.terms.get(&term) {
            self.rows.get_mut(row).unwrap()
        } else {
            self.terms.insert(term, self.rows.len());

            let mut row = BitVec::new();
            row.resize(col + 1, false);
            self.rows.push(row);

            self.rows.last_mut().unwrap()
        };

        row.set(col, true);
    }
}
