use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Ord, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct DocumentId(pub usize);

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub struct TermPositions {
    #[serde(flatten)]
    positions: HashMap<DocumentId, Vec<TermDocumentPosition>>
}

impl TermPositions {
    pub fn new() -> Self {
        TermPositions {
            positions: HashMap::new()
        }
    }

    pub fn documents(&self) -> impl Iterator<Item = DocumentId> + '_ {
        self.positions.keys().cloned()
    }

    pub fn positions_count(&self) -> usize {
        self.positions.values()
            .map(Vec::len)
            .sum()
    }

    pub fn add_position(&mut self, document_id: DocumentId, position: TermDocumentPosition) {
        self.positions.entry(document_id)
            .or_insert_with(Vec::new)
            .push(position);
    }

    pub fn merge(&mut self, mut other: Self) {
        other.positions.drain()
            .for_each(|(document_id, positions)| self.merge_positions(document_id, positions));
    }

    fn merge_positions(&mut self, document_id: DocumentId, positions: Vec<TermDocumentPosition>) {
        self.positions.entry(document_id)
            .or_insert_with(Vec::new)
            .extend(positions);
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct TermDocumentPosition(usize);

impl TermDocumentPosition {
    pub fn new(offset: usize) -> Self {
        TermDocumentPosition(offset)
    }

    pub fn offset(&self) -> usize {
        self.0
    }
}
