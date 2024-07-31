use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::document::DocumentId;
use crate::query_lang::LogicNode;
use crate::position::{TermDocumentPosition, TermPositions};

pub trait TermIndex {
    fn add_term(&mut self, term: String, document_id: DocumentId, position: TermDocumentPosition);
    fn query(&self, query_ast: &LogicNode) -> Result<HashSet<DocumentId>>;
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct InvertedIndex {
    documents: TermPositions,
    index: HashMap<String, TermPositions>
}

impl InvertedIndex {
    pub fn new() -> Self {
        InvertedIndex {
            documents: TermPositions::new(),
            index: HashMap::new()
        }
    }

    pub fn unique_word_count(&self) -> usize {
        self.index.len()
    }

    pub fn total_word_count(&self) -> usize {
        self.index.values()
            .map(TermPositions::positions_count)
            .sum()
    }

    pub fn get_term_positions(&self, term: &str) -> TermPositions {
        self.index.get(term)
            .cloned()
            .unwrap_or_else(TermPositions::new)
    }

    fn documents(&self) -> &TermPositions {
        &self.documents
    }

    pub fn merge(&mut self, mut other: Self) {
        other.index.drain()
            .for_each(|(term, positions)| self.merge_term_positions(term, positions));
    }

    fn merge_term_positions(&mut self, term: String, positions: TermPositions) {
        positions.documents()
            .for_each(|document_id| self.documents.add_document(document_id));

        self.index.entry(term)
            .or_insert_with(TermPositions::new)
            .merge(positions);
    }

    fn query_rec(&self, query_ast: &LogicNode) -> TermPositions {
        match query_ast {
            LogicNode::False => TermPositions::new(),
            LogicNode::Term(term) => self.get_term_positions(term).clone(),
            LogicNode::And(lhs, rhs) => {
                &self.query_rec(lhs) & &self.query_rec(rhs)
            },
            LogicNode::Or(lhs, rhs) => {
                &self.query_rec(lhs) | &self.query_rec(rhs)
            },
            LogicNode::Not(operand) => {
                // NOTE: Not operator works only on document level,
                //  for positions use subtract operator '\'
                self.documents().document_sub(&self.query_rec(&operand))
            },
            LogicNode::Near(lhs, rhs, left, right) => {
                self.query_rec(lhs).close_union(&self.query_rec(rhs), *left, *right)
            },
            LogicNode::Subtract(lhs, rhs) => {
                &self.query_rec(lhs) - &self.query_rec(rhs)
            }
        }
    }
}

impl TermIndex for InvertedIndex {
    fn add_term(&mut self, term: String, document_id: DocumentId, position: TermDocumentPosition) {
        self.index.entry(term)
            .or_insert_with(TermPositions::new)
            .add_position(document_id, position);

        self.documents.add_document(document_id);
    }

    fn query(&self, query_ast: &LogicNode) -> Result<HashSet<DocumentId>> {
        Ok(self.query_rec(query_ast)
            .documents()
            .collect())
    }
}
