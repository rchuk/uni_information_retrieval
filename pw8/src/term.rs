use ahash::{AHashMap, AHashSet};
use crate::document::DocumentId;

#[derive(Eq, PartialEq, Debug)]
pub struct TermPositions {
    positions: AHashMap<DocumentId, usize>
}

impl TermPositions {
    pub fn new() -> Self {
        TermPositions { positions: AHashMap::new() }
    }

    pub fn documents(&self) -> AHashSet<DocumentId> {
        self.positions
            .keys()
            .cloned()
            .collect()
    }

    pub fn document_count(&self) -> usize {
        self.positions.len()
    }

    pub fn count(&self, document_id: DocumentId) -> usize {
        self.positions.get(&document_id)
            .cloned()
            .unwrap_or(0)
    }

    pub fn add_position(&mut self, document_id: DocumentId) {
        self.add_position_with_count(document_id, 1);
    }

    pub fn merge(&mut self, mut other: Self) {
        other.positions.drain()
            .for_each(|(document_id, other_count)| {
                self.add_position_with_count(document_id, other_count);
            });
    }

    pub fn add_position_with_count(&mut self, document_id: DocumentId, delta: usize) {
        self.positions.entry(document_id)
            .and_modify(|count| *count += delta)
            .or_insert(delta);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DocumentId, &usize)> {
        self.positions.iter()
    }
}
