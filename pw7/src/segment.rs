use anyhow::Result;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::document::DocumentId;

#[repr(u8)]
#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash, Debug)]
pub enum SegmentKind {
    Filename = 0,
    Title,
    Authors,
    Body,
    Epigraph
}

impl SegmentKind {
    pub fn values() -> &'static [SegmentKind] {
        &[
            SegmentKind::Filename,
            SegmentKind::Title,
            SegmentKind::Authors,
            SegmentKind::Body,
            SegmentKind::Epigraph
        ]
    }
}

// TODO: Data either should be all owned, or all shared
#[derive(Debug)]
pub struct Segments<'a> {
    segments: HashMap<SegmentKind, Vec<Cow<'a, str>>>
}

impl<'a> Segments<'a> {
    pub fn new() -> Self {
        Segments { segments: HashMap::new() }
    }

    pub fn add(&mut self, segment_kind: SegmentKind, segment: Cow<'a, str>) {
        self.segments.entry(segment_kind)
            .or_insert_with(Vec::new)
            .push(segment)
    }

    pub fn get(&mut self, segment_kind: SegmentKind) -> Option<&Vec<Cow<'a, str>>> {
        self.segments.get(&segment_kind)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SegmentKind, &Vec<Cow<'a, str>>)> {
        self.segments.iter()
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash, Debug)]
pub struct TermPosition {
    pub document: DocumentId,
    pub segment_kind: SegmentKind
}

impl Display for TermPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{:?}]", self.document, self.segment_kind)
    }
}

pub trait Segmenter<'a> {
    fn segment(self: Box<Self>) -> Result<Segments<'a>>;
}
