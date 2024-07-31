use anyhow::{anyhow, Result};
use ahash::{AHashMap, AHashSet};
use std::io::{BufRead, Write};
use std::str::FromStr;
use itertools::Itertools;
use crate::document::DocumentId;
use crate::query_lang::LogicNode;

pub trait TermIndex {
    fn add_term(&mut self, term: String, document_id: DocumentId);
    fn query(&self, query_ast: &LogicNode) -> Result<AHashSet<DocumentId>>;
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct InvertedIndex {
    documents: AHashSet<DocumentId>,
    index: AHashMap<String, AHashSet<DocumentId>>
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

    pub fn term_positions(&self, term: &str) -> AHashSet<DocumentId> {
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

    fn merge_term_positions(&mut self, term: String, positions: AHashSet<DocumentId>) {
        self.documents.extend(&positions);

        self.index.entry(term)
            .or_insert_with(AHashSet::new)
            .extend(positions);
    }

    fn query_rec(&self, query_ast: &LogicNode) -> Result<AHashSet<DocumentId>> {
        Ok(match query_ast {
            LogicNode::False => AHashSet::new(),
            LogicNode::Term(term) => self.term_positions(term),
            LogicNode::And(lhs, rhs) => {
                &self.query_rec(lhs)? & &self.query_rec(rhs)?
            },
            LogicNode::Or(lhs, rhs) => {
                &self.query_rec(lhs)? | &self.query_rec(rhs)?
            },
            LogicNode::Not(operand) => {
                self.documents() - &self.query_rec(&operand)?
            },
            LogicNode::Near(_, _, _, _) => {
                return Err(anyhow!("Operation not supported."));
            },
            LogicNode::Subtract(lhs, rhs) => {
                &self.query_rec(lhs)? - &self.query_rec(rhs)?
            }
        })
    }
}

impl TermIndex for InvertedIndex {
    fn add_term(&mut self, term: String, document_id: DocumentId) {
        self.index.entry(term)
            .or_insert_with(AHashSet::new)
            .insert(document_id);

        self.documents.insert(document_id);
    }

    fn query(&self, query_ast: &LogicNode) -> Result<AHashSet<DocumentId>> {
        self.query_rec(query_ast)
    }
}

impl InvertedIndex {
    const TERM_POSITIONS_SEPARATOR: &'static str = ":";
    const POSITIONS_SEPARATOR: &'static str = ",";

    pub fn save(&self, mut writer: impl Write) -> Result<()> {
        for (term, documents) in &self.index {
            writer.write_all(term.as_bytes())?;
            writer.write_all(Self::TERM_POSITIONS_SEPARATOR.as_bytes())?;
            for (i, document) in documents.iter().enumerate() {
                writer.write_all(format!("{}", document.id()).as_bytes())?;
                if i + 1 != documents.len() {
                    writer.write_all(Self::POSITIONS_SEPARATOR.as_bytes())?;
                }
            }

            writer.write_all("\n".as_bytes())?;
        }

        Ok(())
    }

    pub fn load(reader: impl BufRead) -> Result<Self> {
        let mut index = AHashMap::new();
        for line in reader.lines() {
            let line = line?;
            let (term, positions_str) = line.split(Self::TERM_POSITIONS_SEPARATOR).collect_tuple()
                .ok_or_else(|| anyhow!("Expected term and document ids"))?;
            let mut positions = AHashSet::new();
            for position_str in positions_str.split(Self::POSITIONS_SEPARATOR) {
                let document_id = usize::from_str(position_str)?;

                positions.insert(DocumentId(document_id));
            }

            index.insert(term.to_owned(), positions);
        }

        let documents = index.iter()
            .flat_map(|(_, documents)| documents.iter())
            .cloned()
            .collect();

        Ok(InvertedIndex {
            documents,
            index
        })
    }
}
