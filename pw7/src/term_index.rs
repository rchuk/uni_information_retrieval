use anyhow::{anyhow, Result};
use ahash::{AHashMap, AHashSet};
use std::io::{BufRead, Write};
use std::str::FromStr;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::document::DocumentId;
use crate::query_lang::LogicNode;
use crate::segment::TermPosition;

pub trait TermIndex {
    fn add_term(&mut self, term: String, term_position: TermPosition);
    fn query(&self, query_ast: &LogicNode) -> Result<AHashSet<TermPosition>>;
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct InvertedIndex {
    #[serde(skip)]
    documents: AHashSet<DocumentId>,
    #[serde(flatten)]
    index: AHashMap<String, AHashSet<TermPosition>>
}

impl InvertedIndex {
    pub fn new() -> Self {
        InvertedIndex {
            documents: AHashSet::new(),
            index: AHashMap::new()
        }
    }

    pub fn shrink_to_fit(&mut self) {
        self.documents.shrink_to_fit();
        self.index.shrink_to_fit();
    }

    pub fn unique_word_count(&self) -> usize {
        self.index.len()
    }

    pub fn term_positions(&self, term: &str) -> AHashSet<TermPosition> {
        self.index.get(term)
            .cloned()
            .unwrap_or_else(AHashSet::new)
    }

    fn documents(&self) -> &AHashSet<DocumentId> {
        &self.documents
    }

    pub fn merge(&mut self, mut other: Self) {
        other.index.drain()
            .for_each(|(term, positions)| self.merge_term_positions(term, positions));
    }

    fn merge_term_positions(&mut self, term: String, positions: AHashSet<TermPosition>) {
        self.documents.extend(positions.iter().map(|position| position.document));

        self.index.entry(term)
            .or_insert_with(AHashSet::new)
            .extend(positions);
    }

    fn query_rec(&self, query_ast: &LogicNode) -> Result<AHashSet<TermPosition>> {
        Ok(match query_ast {
            LogicNode::False => AHashSet::new(),
            LogicNode::Term(term) => self.term_positions(term),
            _ => {
                return Err(anyhow!("Operation not supported."));
            }
        })
    }
}

impl TermIndex for InvertedIndex {
    fn add_term(&mut self, term: String, term_position: TermPosition) {
        self.index.entry(term)
            .or_insert_with(AHashSet::new)
            .insert(term_position);

        self.documents.insert(term_position.document);
    }

    fn query(&self, query_ast: &LogicNode) -> Result<AHashSet<TermPosition>> {
        self.query_rec(query_ast)
    }
}
