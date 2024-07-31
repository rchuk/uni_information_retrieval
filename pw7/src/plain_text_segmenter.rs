use std::borrow::Cow;
use anyhow::Result;
use crate::document::DocumentId;
use crate::inf_context::InfContext;
use crate::segment::{Segmenter, SegmentKind, Segments};

pub struct PlainTextSegmenter<'a> {
    document_id: DocumentId,
    ctx: &'a InfContext
}

impl<'a> PlainTextSegmenter<'a> {
    pub fn new(document_id: DocumentId, ctx: &'a InfContext) -> Result<Self> {
        Ok(PlainTextSegmenter {
            document_id,
            ctx
        })
    }
}

impl<'a> Segmenter<'a> for PlainTextSegmenter<'a> {
    fn segment(self: Box<Self>) -> Result<Segments<'a>> {
        let mut segments = Segments::new();

        segments.add(SegmentKind::Body, Cow::Borrowed(self.ctx.document_data(self.document_id)?));

        Ok(segments)
    }
}
