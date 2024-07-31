use std::borrow::Cow;
use anyhow::Result;
use std::sync::Arc;
use crate::inf_context::InfContext;
use crate::term_index::InvertedIndex;
use crate::lexer::{Lexer, LexerStats};
use crate::document::{Document, DocumentId};
use crate::fb2_segmenter::Fb2Segmenter;
use crate::plain_text_segmenter::PlainTextSegmenter;
use crate::segment::{Segmenter, SegmentKind, Segments};

fn get_segmenter(document_id: DocumentId, ctx: &InfContext) -> Result<Box<dyn Segmenter + '_>> {
    if let Some(document) = ctx.document(document_id) {
        if let Document::File { path, .. } = document {
            if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
                return Ok(match extension {
                    "fb2" => Box::new(Fb2Segmenter::new(document_id, ctx)?),
                    _ => Box::new(PlainTextSegmenter::new(document_id, ctx)?)
                });
            }
        }
    }

    Ok(Box::new(PlainTextSegmenter::new(document_id, ctx)?))
}

fn segment_file(document_id: DocumentId, ctx: &InfContext) -> Result<Segments> {
    let segmenter = get_segmenter(document_id, &ctx)?;
    let mut segments = segmenter.segment()?;

    if let Some(document) = ctx.document(document_id) {
        if let Document::File { path, .. } = document {
            path.iter()
                .map(|component| component.to_str())
                .flatten()
                .for_each(|component| segments.add(SegmentKind::Filename, Cow::Owned(component.to_owned())));
        }
    }

    Ok(segments)
}

fn lex_file(document_id: DocumentId, ctx: Arc<InfContext>) -> Result<Option<(InvertedIndex, LexerStats)>> {
    let mut inverted_index = InvertedIndex::new();
    let mut stats = LexerStats::default();
    for (&segment_kind, segments) in segment_file(document_id, &ctx)?.iter() {
        for segment in segments {
            let lexer = Lexer::new(document_id, segment, &ctx)?;
            stats.merge(lexer.lex(&mut inverted_index, segment_kind));
        }
    }
    inverted_index.shrink_to_fit();

    Ok(Some((inverted_index, stats)))
}

pub fn add_file_to_index(document_id: DocumentId, ctx: Arc<InfContext>) -> Result<Option<(InvertedIndex, LexerStats)>> {
    lex_file(document_id, ctx)
}
