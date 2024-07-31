use std::borrow::Cow;
use anyhow::Result;
use fb2::{Author, FictionBook, Section, SectionContent, SectionPart, StyleElement};
use crate::document::DocumentId;
use crate::inf_context::InfContext;
use crate::segment::{Segmenter, SegmentKind, Segments};

pub struct Fb2Segmenter<'a> {
    document_id: DocumentId,
    ctx: &'a InfContext
}

impl<'a> Fb2Segmenter<'a> {
    pub fn new(document_id: DocumentId, ctx: &'a InfContext) -> Result<Self> {
        Ok(Fb2Segmenter {
            document_id,
            ctx
        })
    }

    fn add_sections<'b>(sections: impl Iterator<Item = &'b Section>, segments: &mut Segments) {
        sections
            .map(|section| &section.content)
            .flatten()
            .for_each(|section_content| {
                Self::add_sections(section_content.sections.iter(), segments);

                section_content.content.iter()
                    .for_each(|part| Self::add_section_part(part, segments));
            })
    }

    fn add_section_part(part: &SectionPart, segments: &mut Segments) {
        match part {
            SectionPart::Paragraph(paragraph) => {
                paragraph.elements.iter()
                    .for_each(|element| Self::add_style_element(element, segments));
            },
            _ => ()
        }
    }

    fn add_style_element(element: &StyleElement,  segments: &mut Segments) {
        match element {
            StyleElement::Text(text) => segments.add(SegmentKind::Body, Cow::Owned(text.clone())),
            _ => ()
        }
    }
}

impl<'a> Segmenter<'a> for Fb2Segmenter<'a> {
    fn segment(self: Box<Self>) -> Result<Segments<'a>> {
        let mut segments = Segments::new();

        let data = self.ctx.document_data(self.document_id)?;
        let book = quick_xml::de::from_str::<FictionBook>(data)?;

        segments.add(SegmentKind::Title, Cow::Owned(book.description.title_info.book_title.value));
        book.description.title_info.authors.iter()
            .for_each(|author| match author {
                Author::Verbose(author) => {
                    segments.add(SegmentKind::Authors, Cow::Owned(author.first_name.value.clone()));
                    segments.add(SegmentKind::Authors, Cow::Owned(author.last_name.value.clone()));
                    if let Some(middle_name) = &author.middle_name {
                        segments.add(SegmentKind::Authors, Cow::Owned(middle_name.value.clone()));
                    }
                    if let Some(nickname) = &author.nickname {
                        segments.add(SegmentKind::Authors, Cow::Owned(nickname.value.clone()));
                    }
                },
                Author::Anonymous(author) => {
                    if let Some(nickname) = &author.nickname {
                        segments.add(SegmentKind::Authors, Cow::Owned(nickname.value.clone()));
                    }
                }
            });

        Self::add_sections(book.bodies.iter().flat_map(|body| body.sections.iter()), &mut segments);

        Ok(segments)
    }
}
