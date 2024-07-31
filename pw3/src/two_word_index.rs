use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::document::DocumentId;
use crate::position::TermDocumentPosition;
use crate::query_lang::LogicNode;
use crate::term_index::TermIndex;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct TwoWordIndex {
    #[serde(flatten)]
    index: HashMap<String, HashSet<DocumentId>>,
    #[serde(skip)]
    prev_word: Option<(String, DocumentId)>
}

impl TwoWordIndex {
    pub fn new() -> Self {
        TwoWordIndex {
            index: HashMap::new(),
            prev_word: None
        }
    }

    pub fn unique_word_count(&self) -> usize {
        self.index.len() + 1
    }

    pub fn get_term_documents(&self, term: &str) -> HashSet<DocumentId> {
        self.index.get(term)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }

    fn documents(&self) -> HashSet<DocumentId> {
        self.index.values()
            .flat_map(|documents| documents.iter())
            .cloned()
            .collect()
    }

    pub fn merge(&mut self, mut other: Self) {
        other.index.drain()
            .for_each(|(term, other_documents)| {
                self.index.entry(term)
                    .and_modify(|documents| documents.extend(&other_documents))
                    .or_insert(other_documents);
            });
    }
}

impl TermIndex for TwoWordIndex {
    fn add_term(&mut self, word: String, document_id: DocumentId, _position: TermDocumentPosition) {
        if let Some((prev_word, prev_document_id)) = self.prev_word.take() {
            if prev_document_id == document_id {
                let term = prev_word + "_" + &word;
                self.index.entry(term)
                    .or_insert_with(HashSet::new)
                    .insert(document_id);
            }
        }

        self.prev_word = Some((word, document_id));
    }

    fn query(&self, query_ast: &LogicNode) -> Result<HashSet<DocumentId>> {
        match query_ast {
            LogicNode::False => Ok(HashSet::new()),
            LogicNode::Term(_) => {
                Err(anyhow!("Only 2 word queries are supported."))
            },
            LogicNode::And(lhs, rhs) => {
                Ok(&self.query(lhs)? & &self.query(rhs)?)
            },
            LogicNode::Or(lhs, rhs) => {
                Ok(&self.query(lhs)? | &self.query(rhs)?)
            },
            LogicNode::Not(operand) => {
                Ok(&self.documents() - &self.query(operand)?)
            }
            LogicNode::Subtract(lhs, rhs) => {
                Ok(&self.query(lhs)? - &self.query(rhs)?)
            },
            LogicNode::Near(lhs, rhs, left, right) => {
                if let (LogicNode::Term(lhs), LogicNode::Term(rhs)) = (lhs.as_ref(), rhs.as_ref()) {
                    if *left == 0 && *right == 1 {
                        let term = lhs.to_owned() + "_" + rhs;

                        return Ok(self.get_term_documents(&term));
                    }
                }

                Err(anyhow!("Only 2 word queries are supported."))
            }
        }
    }
}
