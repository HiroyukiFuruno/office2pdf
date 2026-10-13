use std::cell::Cell;
use std::collections::HashMap;
use std::io::{Read, Seek};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use quick_xml::{
    events::{BytesStart, Event},
    name::ResolveResult,
    reader::NsReader,
};

use crate::error::ConvertWarning;
use crate::ir::{DrawingGroup, DrawingGroupAnchor, FrameAnchor, WrapMode};
use crate::parser::units::emu_to_pt;
use crate::parser::xml_util::OOXML_XML_VERSION;

const WORD: &[u8] = b"http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const FRAME: &[u8] = b"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";
const DIAGRAM: &[u8] = b"http://schemas.openxmlformats.org/drawingml/2006/diagram";
const CACHE: &[u8] = b"http://schemas.microsoft.com/office/drawing/2008/diagram";

#[derive(Default)]
pub(super) struct SmartArtContext {
    drawings: Vec<Option<DrawingGroup>>,
    cursor: Cell<usize>,
    pub(super) warnings: Vec<ConvertWarning>,
}

#[derive(Default)]
struct Frame {
    width: f64,
    height: f64,
    effect_extent: crate::ir::Insets,
    data_id: Option<String>,
    anchor: Option<DrawingGroupAnchor>,
    horizontal_position: bool,
}

impl SmartArtContext {
    pub(super) fn consume_next(&self) -> Option<DrawingGroup> {
        let index = self.cursor.get();
        self.cursor.set(index + 1);
        self.drawings.get(index).cloned().flatten()
    }

    pub(super) fn from_archive<R: Read + Seek>(
        xml: Option<&str>,
        theme_xml: Option<&str>,
        archive: &mut zip::ZipArchive<R>,
    ) -> Self {
        let Some(xml) = xml else {
            return Self::default();
        };
        let relationships = super::read_zip_text(archive, "word/_rels/document.xml.rels")
            .map(|xml| relationships(&xml))
            .unwrap_or_default();
        let mut context = Self::default();
        let mut reader = NsReader::from_str(xml);
        let mut depth: usize = 0;
        let mut skipped_depth: Option<usize> = None;
        let mut skipped_hyperlink: bool = false;
        let mut frames: Vec<(usize, usize, Frame)> = Vec::new();
        loop {
            match reader.read_event() {
                Ok(event @ (Event::Start(_) | Event::Empty(_))) => {
                    let children = matches!(event, Event::Start(_));
                    let (Event::Start(element) | Event::Empty(element)) = event else {
                        unreachable!()
                    };
                    let (namespace, name) = reader.resolver().resolve_element(element.name());
                    let is_word =
                        matches!(&namespace, ResolveResult::Bound(ns) if ns.as_ref() == WORD);
                    let is_frame =
                        matches!(&namespace, ResolveResult::Bound(ns) if ns.as_ref() == FRAME);
                    let is_diagram =
                        matches!(&namespace, ResolveResult::Bound(ns) if ns.as_ref() == DIAGRAM);
                    // Keep cache occurrences aligned with the drawings reached
                    // by paragraph conversion, including final-view revisions.
                    let skip: bool = super::contexts::is_skipped_subtree(name.as_ref())
                        || (is_word
                            && matches!(name.as_ref(), b"del" | b"moveFrom" | b"hyperlink"));
                    if skipped_depth.is_some() || skip {
                        if skipped_hyperlink && is_diagram && name.as_ref() == b"relIds" {
                            tracing::warn!(
                                format = "DOCX",
                                subtree = "hyperlink",
                                outcome = "skipped",
                                "SmartArt in an unsupported hyperlink"
                            );
                            context.warnings.push(ConvertWarning::ParseSkipped {
                                format: "DOCX".into(),
                                reason: "SmartArt inside a hyperlink is not yet rendered".into(),
                            });
                        }
                        if children {
                            if skipped_depth.is_none() {
                                skipped_depth = Some(depth);
                                skipped_hyperlink = is_word && name.as_ref() == b"hyperlink";
                            }
                            depth += 1;
                        }
                        continue;
                    }
                    if is_word && name.as_ref() == b"drawing" {
                        let index = context.drawings.len();
                        context.drawings.push(None);
                        frames.push((depth, index, Frame::default()));
                    } else if let Some((_, _, frame)) = frames.last_mut() {
                        if is_frame {
                            match name.as_ref() {
                                b"anchor" => {
                                    frame.anchor = Some(DrawingGroupAnchor {
                                        offset_x: 0.0,
                                        offset_y: 0.0,
                                        horizontal_anchor: FrameAnchor::Text,
                                        vertical_anchor: FrameAnchor::Text,
                                        wrap_mode: WrapMode::None,
                                    })
                                }
                                b"extent" => {
                                    frame.width = number(&element, b"cx");
                                    frame.height = number(&element, b"cy");
                                }
                                b"effectExtent" => {
                                    frame.effect_extent = crate::ir::Insets {
                                        left: number(&element, b"l"),
                                        top: number(&element, b"t"),
                                        right: number(&element, b"r"),
                                        bottom: number(&element, b"b"),
                                    };
                                }
                                b"positionH" | b"positionV" => {
                                    frame.horizontal_position = name.as_ref() == b"positionH";
                                    if let Some(anchor) = frame.anchor.as_mut() {
                                        let relative =
                                            match attribute(&element, b"relativeFrom").as_deref() {
                                                Some("page") => FrameAnchor::Page,
                                                Some("margin") => FrameAnchor::Margin,
                                                _ => FrameAnchor::Text,
                                            };
                                        if frame.horizontal_position {
                                            anchor.horizontal_anchor = relative;
                                        } else {
                                            anchor.vertical_anchor = relative;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        } else if is_diagram && name.as_ref() == b"relIds" {
                            frame.data_id = attribute(&element, b"dm");
                        }
                    }
                    if children {
                        depth += 1;
                    }
                }
                Ok(Event::Text(text)) => {
                    if skipped_depth.is_none()
                        && let Some((_, _, frame)) = frames.last_mut()
                        && let Some(anchor) = frame.anchor.as_mut()
                        && let Ok(value) = text.decode()
                        && let Ok(emu) = value.parse::<i64>()
                    {
                        if frame.horizontal_position {
                            anchor.offset_x = emu_to_pt(emu);
                        } else {
                            anchor.offset_y = emu_to_pt(emu);
                        }
                    }
                }
                Ok(Event::End(_)) => {
                    depth = depth.saturating_sub(1);
                    if skipped_depth.is_some() {
                        if skipped_depth == Some(depth) {
                            skipped_depth = None;
                            skipped_hyperlink = false;
                        }
                        continue;
                    }
                    if frames.last().is_some_and(|(start, _, _)| *start == depth) {
                        let (_, index, frame) = frames.pop().unwrap();
                        if frame.data_id.is_some() {
                            context.drawings[index] = load_frame(
                                frame,
                                &relationships,
                                theme_xml,
                                archive,
                                &mut context.warnings,
                            );
                        }
                    }
                }
                Ok(Event::Eof) | Err(_) => break,
                _ => {}
            }
        }
        context
    }
}

fn attribute(element: &BytesStart<'_>, name: &[u8]) -> Option<String> {
    element.attributes().flatten().find_map(|attribute| {
        (attribute.key.local_name().as_ref() == name)
            .then(|| attribute.normalized_value(OOXML_XML_VERSION).ok())
            .flatten()
            .map(|value| value.into_owned())
    })
}

fn number(element: &BytesStart<'_>, name: &[u8]) -> f64 {
    attribute(element, name)
        .and_then(|value| value.parse::<i64>().ok())
        .map(emu_to_pt)
        .unwrap_or_default()
}

fn relationships(xml: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element))
                if element.local_name().as_ref() == b"Relationship" =>
            {
                if let (Some(id), Some(target)) =
                    (attribute(&element, b"Id"), attribute(&element, b"Target"))
                {
                    result.insert(id, target);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    result
}

fn part_path(target: &str) -> String {
    let mut parts: Vec<&str> = if target.starts_with('/') {
        Vec::new()
    } else {
        vec!["word"]
    };
    for part in target.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

fn load_frame<R: Read + Seek>(
    frame: Frame,
    relationships: &HashMap<String, String>,
    theme_xml: Option<&str>,
    archive: &mut zip::ZipArchive<R>,
    warnings: &mut Vec<ConvertWarning>,
) -> Option<DrawingGroup> {
    let started = Instant::now();
    let recovered = (|| {
        let data_path = part_path(relationships.get(frame.data_id.as_ref()?)?);
        let data_xml = super::read_zip_text(archive, &data_path)?;
        let mut reader = NsReader::from_str(&data_xml);
        let mut drawing_id = None;
        loop {
            match reader.read_event() {
                Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                    let (ns, name) = reader.resolver().resolve_element(element.name());
                    if matches!(ns, ResolveResult::Bound(ns) if ns.as_ref() == CACHE)
                        && name.as_ref() == b"dataModelExt"
                    {
                        drawing_id = attribute(&element, b"relId");
                        break;
                    }
                }
                Ok(Event::Eof) | Err(_) => break,
                _ => {}
            }
        }
        let drawing_path = part_path(relationships.get(&drawing_id?)?);
        let (elements, parser_warnings) =
            super::super::pptx::parse_cached_drawing_part(archive, &drawing_path, theme_xml)
                .ok()?;
        for mut warning in parser_warnings {
            match &mut warning {
                ConvertWarning::UnsupportedElement { format, .. }
                | ConvertWarning::PartialElement { format, .. }
                | ConvertWarning::FallbackUsed { format, .. }
                | ConvertWarning::ParseSkipped { format, .. } => *format = "DOCX".into(),
            }
            warnings.push(warning);
        }
        if elements.is_empty() || frame.width <= 0.0 || frame.height <= 0.0 {
            return None;
        }
        Some(DrawingGroup {
            width: frame.width,
            height: frame.height,
            effect_extent: frame.effect_extent,
            elements,
            anchor: frame.anchor,
        })
    })();
    if recovered.is_none() {
        tracing::warn!(
            format = "DOCX",
            elapsed_ms = started.elapsed().as_millis(),
            outcome = "skipped",
            "SmartArt drawing cache could not be loaded"
        );
        warnings.push(ConvertWarning::ParseSkipped {
            format: "DOCX".into(),
            reason: "SmartArt drawing cache could not be loaded".into(),
        });
    }
    recovered
}
