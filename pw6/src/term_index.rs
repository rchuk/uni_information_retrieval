use anyhow::{anyhow, Result};
use ahash::{AHashMap, AHashSet};
use std::io::{BufRead, Write};
use std::iter::Peekable;
use std::str::FromStr;
use itertools::Itertools;
use crate::document::DocumentId;
use crate::query_lang::LogicNode;
use crate::encoding::{vb_decode, vb_encode};

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

    pub fn save_compressed(&self, mut writer: impl Write) -> Result<()> {
        let terms = self.write_dictionary_compressed(&mut writer)?;

        for documents in terms.iter().map(|&term| self.index.get(term).unwrap()) {
            let mut prev_document_id = 0;

            let documents_count = documents.len();
            writer.write_all(&vb_encode(documents_count))?;
            for document in documents.iter().sorted() {
                let delta = document.id() - prev_document_id;
                prev_document_id = document.id();

                let delta_vb = vb_encode(delta);
                writer.write_all(&delta_vb)?;
            }
        }

        Ok(())
    }

    pub fn read_compressed(reader: impl BufRead) -> Result<Self> {
        let mut iter = reader.bytes().peekable();

        let mut terms = Self::read_dictionary_compressed(&mut iter)?;
        let mut index = AHashMap::with_capacity(terms.len());
        for term in terms.drain(..) {
            let document_count = vb_decode(&mut iter)?;
            let mut documents = AHashSet::with_capacity(document_count);
            let mut prev_document_id = 0;
            for _ in 0..document_count {
                let delta = vb_decode(&mut iter)?;
                prev_document_id += delta;

                documents.insert(DocumentId(prev_document_id));
            }

            index.insert(term, documents);
        }

        let documents = index.iter()
            .flat_map(|(_, documents)| documents.iter())
            .cloned()
            .collect();

        Ok(InvertedIndex {
            index,
            documents
        })
    }

    fn write_dictionary_compressed(&self, writer: &mut impl Write) -> Result<Vec<&String>> {
        let mut anchor = None;
        let terms: Vec<&String> = self.index.keys().sorted().collect();
        for term in terms.iter() {
            let prefix_len = if let Some(anchor) = anchor {
                Self::longest_prefix(anchor, term)
            } else {
                0
            };

            anchor = Some(term);
            writer.write_all(format!("{}", prefix_len).as_bytes())?;
            writer.write_all(term[prefix_len..].as_bytes())?;
        }
        writer.write_all(&[0u8])?;

        Ok(terms)
    }

    fn read_dictionary_compressed(iter: &mut Peekable<impl Iterator<Item = Result<u8, std::io::Error>>>) -> Result<Vec<String>> {
        let mut terms = Vec::<String>::new();

        while let Some(&Ok(byte)) = iter.peek() {
            if byte == 0u8 {
                iter.next();
                break;
            }

            let prefix_len = Self::read_number(iter)?;
            let text = Self::read_text(iter)?;

            if let Some(anchor) = terms.last() {
                terms.push(anchor[..prefix_len].to_owned() + &text);
            } else {
                terms.push(text);
            }
        }

        Ok(terms)
    }

    fn read_number(iter: &mut Peekable<impl Iterator<Item = Result<u8, std::io::Error>>>) -> Result<usize> {
        let mut number_str = String::new();
        while let Some(&Ok(byte)) = iter.peek() {
            if !byte.is_ascii_digit() {
                break;
            }

            number_str.push(byte as char);
            iter.next();
        }

        Ok(number_str.parse()?)
    }

    fn read_text(iter: &mut Peekable<impl Iterator<Item = Result<u8, std::io::Error>>>) -> Result<String> {
        let mut buf = Vec::new();
        while let Some(&Ok(byte)) = iter.peek() {
            if byte == 0u8 || byte.is_ascii_digit() {
                break;
            }

            buf.push(byte);
            iter.next();
        }

        Ok(String::from_utf8(buf)?)
    }

    fn longest_prefix(anchor: &str, term: &str) -> usize {
        anchor
            .char_indices()
            .zip(term.chars())
            .find(|((_, c1), c2)| c1 != c2)
            .map(|((i, _), _)| i)
            .unwrap_or_else(|| anchor.len())
    }
}
