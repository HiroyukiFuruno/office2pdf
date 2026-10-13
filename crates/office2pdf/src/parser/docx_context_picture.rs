use std::cell::Cell;

use quick_xml::NsReader;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;

use crate::ir::ImageCrop;
use crate::parser::xml_util::OOXML_XML_VERSION;

const WORD: &[u8] = b"http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const PICTURE: &[u8] = b"http://schemas.openxmlformats.org/drawingml/2006/picture";
const DRAWING: &[u8] = b"http://schemas.openxmlformats.org/drawingml/2006/main";

#[derive(Default)]
struct DrawingCrop {
    crop: Option<ImageCrop>,
    subtree_end: usize,
}

/// The released docx-rs does not expose source rectangles. Keep metadata per
/// drawing occurrence, not per media relationship, which pictures can share.
#[derive(Default)]
pub(in super::super) struct PictureCropContext {
    drawings: Vec<DrawingCrop>,
    cursor: Cell<usize>,
}

impl PictureCropContext {
    pub(in super::super) fn from_xml(xml: Option<&str>) -> Self {
        Self {
            drawings: xml.map(scan_picture_crops).unwrap_or_default(),
            cursor: Cell::new(0),
        }
    }

    /// Header text-box children use a separate conversion path;
    /// skip their nested drawing entries when the ordinary path does not visit them.
    pub(in super::super) fn next_drawing(&self, skip_nested: bool) -> Option<ImageCrop> {
        let index: usize = self.cursor.get();
        let drawing: &DrawingCrop = self.drawings.get(index)?;
        self.cursor.set(if skip_nested {
            drawing.subtree_end
        } else {
            index + 1
        });
        drawing.crop
    }
}

fn namespace(reader: &NsReader<&[u8]>, element: &BytesStart<'_>) -> Vec<u8> {
    match reader.resolver().resolve_element(element.name()).0 {
        ResolveResult::Bound(value) => value.as_ref().to_vec(),
        _ => Vec::new(),
    }
}

fn source_rectangle(reader: &NsReader<&[u8]>, element: &BytesStart<'_>) -> Option<ImageCrop> {
    let mut crop: ImageCrop = ImageCrop::default();
    for attribute in element.attributes().flatten() {
        // Source-rectangle attributes are unqualified; alien namespaced
        // attributes must not override the four DrawingML values.
        if attribute.key.prefix().is_some() {
            continue;
        }
        let Ok(value) = attribute.decoded_and_normalized_value(OOXML_XML_VERSION, reader.decoder())
        else {
            continue;
        };
        let value: &str = value.trim();
        let offset: Option<f64> = if let Some(percent) = value.strip_suffix('%') {
            percent
                .trim()
                .parse::<f64>()
                .ok()
                .map(|number| number / 100.0)
        } else {
            value
                .parse::<i32>()
                .ok()
                .map(|number| f64::from(number) / 100_000.0)
        };
        let Some(offset) = offset.filter(|number| number.is_finite()) else {
            continue;
        };
        match attribute.key.as_ref() {
            b"l" => crop.left = offset,
            b"t" => crop.top = offset,
            b"r" => crop.right = offset,
            b"b" => crop.bottom = offset,
            _ => {}
        }
    }
    (!crop.is_empty()).then_some(crop)
}

fn scan_picture_crops(xml: &str) -> Vec<DrawingCrop> {
    let mut reader: NsReader<&[u8]> = NsReader::from_str(xml);
    let mut path: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    let mut drawings: Vec<DrawingCrop> = Vec::new();
    let mut open_drawings: Vec<(usize, usize)> = Vec::new();
    let mut skipped_depth: Option<usize> = None;

    loop {
        let (element, empty) = match reader.read_event() {
            Ok(Event::Start(element)) => (element, false),
            Ok(Event::Empty(element)) => (element, true),
            Ok(Event::End(_)) => {
                if open_drawings
                    .last()
                    .is_some_and(|(_, depth)| *depth == path.len())
                {
                    let (index, _) = open_drawings.pop().unwrap();
                    drawings[index].subtree_end = drawings.len();
                }
                if skipped_depth == Some(path.len()) {
                    skipped_depth = None;
                }
                path.pop();
                continue;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => continue,
        };
        let uri: Vec<u8> = namespace(&reader, &element);
        let local: Vec<u8> = element.local_name().as_ref().to_vec();
        let skip: bool = super::paragraph_cursor::is_skipped_subtree(&local)
            || (uri == WORD && matches!(local.as_slice(), b"del" | b"moveFrom" | b"hyperlink"));
        if skipped_depth.is_none() && skip && !empty {
            skipped_depth = Some(path.len() + 1);
        }
        if skipped_depth.is_none() && !skip {
            if uri == WORD && local == b"drawing" {
                let index: usize = drawings.len();
                drawings.push(DrawingCrop {
                    crop: None,
                    subtree_end: index + 1,
                });
                if !empty {
                    open_drawings.push((index, path.len() + 1));
                }
            } else if uri == DRAWING
                && local == b"srcRect"
                && path.len() >= 2
                && path[path.len() - 1].0 == PICTURE
                && path[path.len() - 1].1 == b"blipFill"
                && path[path.len() - 2].0 == PICTURE
                && path[path.len() - 2].1 == b"pic"
                && let Some((index, _)) = open_drawings.last()
            {
                drawings[*index].crop = source_rectangle(&reader, &element);
            }
        }
        if !empty {
            path.push((uri, local));
        }
    }
    drawings
}
