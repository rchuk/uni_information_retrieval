use std::collections::{BTreeSet, HashMap};
use std::ops::{BitAnd, BitOr, Sub};
use std::ops::Bound::Included;
use serde::{Deserialize, Serialize};
use crate::document::DocumentId;

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub struct TermPositions {
    #[serde(flatten)]
    positions: HashMap<DocumentId, BTreeSet<TermDocumentPosition>>
}

impl TermPositions {
    pub fn new() -> Self {
        TermPositions {
            positions: HashMap::new()
        }
    }

    pub fn with_positions(positions: HashMap<DocumentId, BTreeSet<TermDocumentPosition>>) -> Self {
        TermPositions { positions }
    }

    pub fn documents(&self) -> impl Iterator<Item = DocumentId> + '_ {
        self.positions.keys().cloned()
    }

    pub fn positions_count(&self) -> usize {
        self.positions.values()
            .map(BTreeSet::len)
            .sum()
    }

    pub fn add_document(&mut self, document_id: DocumentId) {
        self.positions.entry(document_id)
            .or_insert_with(BTreeSet::new);
    }

    pub fn add_position(&mut self, document_id: DocumentId, position: TermDocumentPosition) {
        self.positions.entry(document_id)
            .or_insert_with(BTreeSet::new)
            .insert(position);
    }

    pub fn merge(&mut self, mut other: Self) {
        other.positions.drain()
            .for_each(|(document_id, positions)| self.merge_positions(document_id, positions));
    }

    pub fn close_union(&self, other: &Self, left: usize, right: usize) -> TermPositions {
        let result = self.positions.iter()
            .flat_map(|(&document_id, positions)| {
                other.positions.get(&document_id)
                    .map(|other_positions| (document_id, positions, other_positions))
            })
            .map(|(document_id, positions, other_positions)| {
                (
                    document_id,
                    positions.iter()
                        .flat_map(|&position| Self::positions_around_and_self(other_positions, position, left, right).into_iter())
                        .collect::<BTreeSet<TermDocumentPosition>>()
                )
            })
            .filter(|(_, positions)| !positions.is_empty())
            .collect();

        TermPositions::with_positions(result)
    }

    fn positions_around_and_self(positions: &BTreeSet<TermDocumentPosition>, position: TermDocumentPosition, left: usize, right: usize) -> BTreeSet<TermDocumentPosition> {
        let mut result: BTreeSet<TermDocumentPosition> = Self::positions_around(positions, position, left, right).cloned().collect();
        if !result.is_empty() {
            result.insert(position);
        }

        result
    }

    fn positions_around(positions: &BTreeSet<TermDocumentPosition>, position: TermDocumentPosition, left: usize, right: usize) -> impl Iterator<Item = &TermDocumentPosition> {
        let min = TermDocumentPosition(position.offset().saturating_sub(left));
        let max = TermDocumentPosition(position.offset().saturating_add(right));

        positions.range((Included(min), Included(max)))
    }

    fn merge_positions(&mut self, document_id: DocumentId, positions: BTreeSet<TermDocumentPosition>) {
        self.positions.entry(document_id)
            .or_insert_with(BTreeSet::new)
            .extend(positions);
    }

    pub fn document_sub(&self, rhs: &TermPositions) -> TermPositions {
        let result = self.positions.iter()
            .filter(|(document_id, _)| !rhs.positions.contains_key(document_id))
            .map(|(&document_id, positions)| (document_id, positions.clone()))
            .collect();

        TermPositions::with_positions(result)
    }
}

impl BitOr<&TermPositions> for &TermPositions {
    type Output = TermPositions;

    fn bitor(self, rhs: &TermPositions) -> Self::Output {
        let mut result = HashMap::new();
        let mut bitor = |pos: &HashMap<DocumentId, BTreeSet<TermDocumentPosition>>| {
            pos.iter().for_each(|(&document_id, positions)| {
                result.entry(document_id)
                    .or_insert_with(BTreeSet::new)
                    .extend(positions);
            });
        };
        bitor(&self.positions);
        bitor(&rhs.positions);

        TermPositions::with_positions(result)
    }
}

impl BitAnd<&TermPositions> for &TermPositions {
    type Output = TermPositions;

    fn bitand(self, rhs: &TermPositions) -> Self::Output {
        let result = self.positions.iter()
            .filter_map(|(&document_id, positions)| {
                rhs.positions.get(&document_id)
                    .map(|other_positions| (document_id, positions & other_positions))
            })
            .collect();

        TermPositions::with_positions(result)
    }
}

impl Sub<&TermPositions> for &TermPositions {
    type Output = TermPositions;

    fn sub(self, rhs: &TermPositions) -> Self::Output {
        let result = self.positions.iter()
            .map(|(&document_id, positions)| {
                (
                    document_id,
                    rhs.positions.get(&document_id)
                        .map(|other_positions| positions - other_positions)
                        .unwrap_or_else(|| positions.clone())
                )
            })
            .filter(|(_, positions)| !positions.is_empty())
            .collect();

        TermPositions::with_positions(result)
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct TermDocumentPosition(usize);

impl TermDocumentPosition {
    pub fn new(offset: usize) -> Self {
        TermDocumentPosition(offset)
    }

    pub fn offset(&self) -> usize {
        self.0
    }
}
