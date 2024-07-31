use std::collections::BTreeMap;
use anyhow::{anyhow, Result};
use ahash::{AHashMap, AHashSet};
use std::io::{BufRead, Write};
use std::str::FromStr;
use itertools::Itertools;
use nalgebra::DVector;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use crate::document::DocumentId;
use crate::term::TermPositions;

pub trait TermIndex {
    fn add_term(&mut self, term: String, document_id: DocumentId);
    fn query(&self, terms: &AHashSet<String>, leader_count: usize) -> Result<Vec<(DocumentId, f64)>>;
}

#[derive(Debug)]
pub struct InvertedIndex {
    documents: AHashMap<DocumentId, usize>,
    index: BTreeMap<String, TermPositions>,
    vectors: AHashMap<DocumentId, DVector<f64>>,
    leaders: AHashSet<DocumentId>,
    followers: AHashMap<DocumentId, Vec<DocumentId>>
}

impl InvertedIndex {
    pub fn new() -> Self {
        InvertedIndex {
            documents: AHashMap::new(),
            index: BTreeMap::new(),
            vectors: AHashMap::new(),
            leaders: AHashSet::new(),
            followers: AHashMap::new()
        }
    }

    pub fn preprocess(&mut self, follower_leader_count: usize) {
        let leader_count = (self.documents.len() as f64).sqrt() as usize;
        let mut documents = self.documents.keys()
            .cloned()
            .collect::<Vec<_>>();
        documents.shuffle(&mut thread_rng());
        let (leader_ids, follower_ids) = documents.split_at(leader_count);

        self.vectors = self.documents.keys()
            .map(|&document_id| (document_id, self.document_tf_idf(document_id)))
            .collect();

        self.leaders = leader_ids.iter().cloned().collect();

        let followers_to_leaders = follower_ids.iter()
            .map(|&follower| {
                (
                    follower,
                    self.closest_documents(follower_leader_count, &self.vectors[&follower], self.leaders.iter())
                        .iter()
                        .map(|(document_id, _)| *document_id)
                        .collect::<Vec<_>>()
                )
            })
            .collect::<AHashMap<_, _>>();

        self.followers = followers_to_leaders.iter()
            .flat_map(|(follower, leaders)| {
                leaders.iter()
                    .map(|leader| (*leader, *follower))
            })
            .sorted_by_key(|(leader, _)| *leader)
            .group_by(|(leader, _)| *leader)
            .into_iter()
            .map(|(leader, group)|
                (
                    leader,
                    group.into_iter()
                        .map(|(_, follower)| follower)
                        .collect::<Vec<_>>()
                )
            )
            .collect();
    }

    pub fn shrink_to_fit(&mut self) {
        self.documents.shrink_to_fit();
    }

    pub fn term_count(&self) -> usize {
        self.index.len()
    }

    fn closest_documents<'a>(&self, count: usize, needle: &DVector<f64>, haystack: impl Iterator<Item = &'a DocumentId>)
        -> Vec<(DocumentId, f64)> {
        haystack
            .map(|&document_id| (document_id, Self::cosine_sim(&self.vectors[&document_id], needle)))
            .sorted_by(|(_, sim_a), (_, sim_b)| sim_a.partial_cmp(sim_b).unwrap())
            .take(count)
            .collect()
    }

    fn cosine_sim(a: &DVector<f64>, b: &DVector<f64>) -> f64 {
        let a_mag = a.magnitude();
        let b_mag = b.magnitude();
        if a_mag == 0.0 || b_mag == 0.0 {
            return 0.0;
        }

        a.dot(b) / (a_mag * b_mag)
    }

    fn document_tf_idf(&self, document_id: DocumentId) -> DVector<f64> {
        self.terms_frequency(document_id).component_mul(&self.inverse_document_frequency())
    }

    fn terms_frequency(&self, document_id: DocumentId) -> DVector<f64> {
        let document_term_count = self.documents.get(&document_id).cloned().unwrap_or(0) as f64;

        self.terms_count(document_id) / document_term_count
    }

    fn terms_count(&self, document_id: DocumentId) -> DVector<f64> {
        DVector::from_iterator(
            self.term_count(),
            self.index.values()
                .map(|positions| positions.count(document_id) as f64)
        )
    }

    fn inverse_document_frequency(&self) -> DVector<f64> {
        let total_count = self.documents.len() as f64;
        let mut vector = DVector::from_iterator(
            self.term_count(),
            self.index.values()
                .map(|positions| positions.document_count() as f64)
        );

        vector.add_scalar_mut(1.0);
        vector.apply(|x| *x = 1.0 / *x);
        vector *= total_count + 1.0;
        vector.apply(|x| *x = x.log2());

        vector
    }

    fn query_vector(&self, terms: &AHashSet<String>) -> DVector<f64> {
        DVector::from_iterator(
            self.term_count(),
            self.index.keys()
                .map(|term| terms.contains(term).then_some(1.0).unwrap_or(0.0))
        )
    }

    pub fn term_documents(&self, term: &str) -> AHashSet<DocumentId> {
        self.index.get(term)
            .map(|positions| positions.documents())
            .unwrap_or_else(AHashSet::new)
    }

    pub fn document_term_count(&self, document_id: DocumentId) -> usize {
        self.documents.get(&document_id)
            .cloned()
            .unwrap_or(0)
    }

    fn documents(&self) -> AHashSet<DocumentId> {
        self.documents.keys()
            .cloned()
            .collect()
    }

    pub fn terms(&self) -> AHashSet<String> {
        self.index.keys()
            .cloned()
            .collect()
    }

    pub fn merge(&mut self, mut other: Self) {
        other.documents.drain()
            .for_each(|(document_id, other_count)| {
                self.documents.entry(document_id)
                    .and_modify(|count| *count += other_count)
                    .or_insert(other_count);
            });

        other.index.into_iter()
            .for_each(|(term, other_positions)| {
                self.index.entry(term)
                    .or_insert_with(TermPositions::new)
                    .merge(other_positions);
            });
    }
}

impl TermIndex for InvertedIndex {
    fn add_term(&mut self, term: String, document_id: DocumentId) {
        self.index.entry(term)
            .or_insert_with(TermPositions::new)
            .add_position(document_id);

        self.documents.entry(document_id)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    fn query(&self, terms: &AHashSet<String>, leader_count: usize) -> Result<Vec<(DocumentId, f64)>> {
        let needle = self.query_vector(terms);
        if needle.magnitude_squared() == 0.0 {
            return Err(anyhow!("Index doesn't contain any word from the query"));
        }

        let leaders = self.closest_documents(leader_count, &needle, self.leaders.iter());
        let followers = leaders.iter()
            .flat_map(|(leader, _)|
                self.followers.get(leader).iter()
                    .flat_map(|followers| {
                        followers.iter()
                            .map(|&follower| (follower, Self::cosine_sim(&needle, &self.vectors[&follower])))
                    })
                    .collect::<Vec<_>>()
            );

        Ok(leaders.iter()
            .cloned()
            .chain(followers)
            .sorted_by(|(_, sim_a), (_, sim_b)| sim_a.partial_cmp(sim_b).unwrap().reverse())
            .collect())
    }
}

impl InvertedIndex {
    const TERM_POSITIONS_SEPARATOR: &'static str = "|";
    const KEY_VALUE_SEPARATOR: &'static str = ":";
    const VALUE_SEPARATOR: &'static str = ",";
    const DOCUMENT_POSITIONS_SEPARATOR: &'static str = "#";

    pub fn save(&self, mut writer: impl Write) -> Result<()> {
        for (document, count) in self.documents.iter().sorted_by_key(|(&document_id, _)| document_id) {
            writer.write_all(format!("{}{}{}\n", document.id(), Self::KEY_VALUE_SEPARATOR, count).as_bytes())?;
        }
        writer.write_all(format!("{}\n", Self::DOCUMENT_POSITIONS_SEPARATOR).as_bytes())?;

        for (term, positions) in &self.index {
            writer.write_all(term.as_bytes())?;
            writer.write_all(Self::TERM_POSITIONS_SEPARATOR.as_bytes())?;
            for (i, (document, count)) in positions.iter().enumerate() {
                if i != 0 {
                    writer.write_all(Self::VALUE_SEPARATOR.as_bytes())?;
                }
                writer.write_all(format!("{}{}{}", document.id(), Self::KEY_VALUE_SEPARATOR, count).as_bytes())?;
            }

            writer.write_all("\n".as_bytes())?;
        }

        Ok(())
    }

    pub fn load(reader: impl BufRead) -> Result<Self> {
        let mut index = InvertedIndex::new();

        let mut iter = reader.lines();
        Self::read_documents(&mut index, &mut iter)?;
        Self::read_positions(&mut index, &mut iter)?;

        Ok(index)
    }

    fn read_documents(index: &mut Self, iter: &mut impl Iterator<Item = Result<String, std::io::Error>>) -> Result<()> {
        for line in iter {
            let line = line?;
            if line == Self::DOCUMENT_POSITIONS_SEPARATOR {
                break;
            }

            Self::read_documents_line(index, &line)?;
        }

        Ok(())
    }

    fn read_positions(index: &mut Self, iter: &mut impl Iterator<Item = Result<String, std::io::Error>>) -> Result<()> {
        for line in iter {
            let line = line?;

            Self::read_positions_line(index, &line)?;
        }

        Ok(())
    }

    fn read_documents_line(index: &mut Self, line: &str) -> Result<()> {
        let (document_str, count_str) = line.split(Self::KEY_VALUE_SEPARATOR).collect_tuple()
            .ok_or_else(|| anyhow!("Expected document id and term count"))?;

        let document = usize::from_str(document_str)?;
        let count = usize::from_str(count_str)?;

        index.documents.insert(DocumentId(document), count);

        Ok(())
    }

    fn read_positions_line(index: &mut Self, line: &str) -> Result<()> {
        let (term, positions_str) = line.split(Self::TERM_POSITIONS_SEPARATOR).collect_tuple()
            .ok_or_else(|| anyhow!("Expected term and document ids"))?;
        let mut positions = TermPositions::new();
        for position_str in positions_str.split(Self::VALUE_SEPARATOR) {
            let (document_str, count_str) = position_str.split(Self::KEY_VALUE_SEPARATOR).collect_tuple()
                .ok_or_else(|| anyhow!("Expected document and count"))?;

            let document = usize::from_str(document_str)?;
            let count = usize::from_str(count_str)?;

            positions.add_position_with_count(DocumentId(document), count);
        }

        index.index.insert(term.to_owned(), positions);

        Ok(())
    }
}
