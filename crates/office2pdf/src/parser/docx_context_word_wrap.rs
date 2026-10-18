//! Per-paragraph `w:wordWrap` recovered from the raw document XML.
//!
//! Upstream `docx-rs` does not parse `w:pPr/w:wordWrap`. When the fork that
//! does was patched in through `[patch.crates-io]`, which never reaches a
//! published crate, reading its field made the crate unpublishable — the
//! v0.6.6 release's crates.io job failed on exactly that (issue #1041).
//! Scanning the raw XML keeps the behaviour of issue #730 independent of
//! which `docx-rs` build is linked, even though the `office2pdf-docx-rs`
//! dependency now parses the element too.
//!
//! The scan counts the paragraph sequence of [`super::paragraph_cursor`], so a
//! text box's `mc:Fallback` copy cannot shift the property onto a later
//! paragraph (issue #1689).

use std::collections::HashMap;

use crate::parser::xml_util::OOXML_XML_VERSION;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use super::paragraph_cursor::{
    ParagraphCursor, ScannedParagraph, attribute_value, scan_body_paragraphs,
};

/// `w:wordWrap`'s `w:val`, with the ST_OnOff spellings Word writes. An absent
/// attribute means "on", the element's own default.
fn word_wrap_value(reader: &Reader<&[u8]>, element: &BytesStart<'_>) -> Option<bool> {
    match attribute_value(reader, element, b"val").as_deref() {
        Some("0") | Some("false") | Some("off") => Some(false),
        _ => Some(true),
    }
}

pub(in super::super) struct WordWrapContext {
    values: ParagraphCursor<Option<bool>>,
}

impl WordWrapContext {
    pub(in super::super) fn from_xml(xml: Option<&str>) -> Self {
        Self {
            values: ParagraphCursor::new("w:wordWrap", xml.map(Self::scan).unwrap_or_default()),
        }
    }

    /// The next paragraph's `w:wordWrap`, `None` when it states none. Must be
    /// called exactly once per converted `w:p`, with that paragraph's
    /// `w:pStyle`.
    pub(in super::super) fn next_word_wrap(&self, style_id: Option<&str>) -> Option<bool> {
        self.values
            .next(style_id)
            .and_then(|(_, word_wrap)| *word_wrap)
    }

    fn scan(xml: &str) -> Vec<ScannedParagraph<Option<bool>>> {
        scan_body_paragraphs(xml, |reader, element, word_wrap| {
            if element.local_name().as_ref() == b"wordWrap" {
                *word_wrap = word_wrap_value(reader, element);
            }
        })
    }
}

/// Each paragraph style's `w:wordWrap` from `styles.xml`, by style id, for the
/// style chain of issue #730.
pub(in super::super) fn scan_style_word_wrap(xml: Option<&str>) -> HashMap<String, bool> {
    let Some(xml) = xml else {
        return HashMap::new();
    };
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut map: HashMap<String, bool> = HashMap::new();
    let mut current_style: Option<String> = None;
    let mut in_paragraph_properties = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                match element.local_name().as_ref() {
                    b"style" => {
                        // Paragraph styles only, like the shading scanner: a
                        // stray `w:wordWrap` on a character style must not
                        // leak into the paragraph-style chain.
                        let mut style_id: Option<String> = None;
                        let mut style_type: Option<String> = None;
                        for attribute in element.attributes().flatten() {
                            let value = attribute
                                .decoded_and_normalized_value(OOXML_XML_VERSION, reader.decoder())
                                .ok()
                                .map(|value| value.into_owned());
                            match attribute.key.local_name().as_ref() {
                                b"styleId" => style_id = value,
                                b"type" => style_type = value,
                                _ => {}
                            }
                        }
                        current_style =
                            style_id.filter(|_| style_type.as_deref() == Some("paragraph"));
                    }
                    b"pPr" if current_style.is_some() => in_paragraph_properties = true,
                    b"wordWrap" if in_paragraph_properties => {
                        if let (Some(style_id), Some(value)) =
                            (current_style.clone(), word_wrap_value(&reader, &element))
                        {
                            map.insert(style_id, value);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(element)) => match element.local_name().as_ref() {
                b"style" => current_style = None,
                b"pPr" => in_paragraph_properties = false,
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    map
}
