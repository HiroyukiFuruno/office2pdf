use crate::ir::{
    BorderLineStyle, BorderSide, Color, LineCap, LineJoin, PageBorderDisplay, PageBorders,
};
use crate::parser::xml_util::{OOXML_XML_VERSION, parse_hex_color};
use quick_xml::{
    events::{BytesStart, Event},
    name::ResolveResult,
    reader::NsReader,
};
use std::collections::HashMap;

pub(super) fn is_word(namespace: ResolveResult<'_>) -> bool {
    matches!(namespace, ResolveResult::Bound(n) if matches!(n.as_ref(),
        b"http://schemas.openxmlformats.org/wordprocessingml/2006/main" |
        b"http://purl.oclc.org/ooxml/wordprocessingml/main"))
}

pub(super) fn attributes(
    reader: &NsReader<&[u8]>,
    element: &BytesStart<'_>,
) -> HashMap<String, String> {
    element
        .attributes()
        .flatten()
        .filter_map(|attribute| {
            let (namespace, name) = reader.resolver().resolve_attribute(attribute.key);
            is_word(namespace)
                .then(|| {
                    Some((
                        String::from_utf8_lossy(name.as_ref()).into_owned(),
                        attribute
                            .normalized_value(OOXML_XML_VERSION)
                            .ok()?
                            .into_owned(),
                    ))
                })
                .flatten()
        })
        .collect()
}

fn side(values: &HashMap<String, String>) -> Option<BorderSide> {
    let style = match values.get("val").map(String::as_str)? {
        "single" | "thick" => BorderLineStyle::Solid,
        "dashed" | "dashSmallGap" => BorderLineStyle::Dashed,
        "dotted" => BorderLineStyle::Dotted,
        "dotDash" => BorderLineStyle::DashDot,
        "dotDotDash" => BorderLineStyle::DashDotDot,
        "double" => BorderLineStyle::Double,
        // Art borders must not become invented straight lines.
        "none" | "nil" => return None,
        other => {
            tracing::warn!(style = other, "Unsupported DOCX page-border style");
            return None;
        }
    };
    let width: f64 = values
        .get("sz")
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| value / 8.0)
        .unwrap_or(0.5);
    if !width.is_finite() || width <= 0.0 {
        return None;
    }
    Some(BorderSide {
        width,
        color: values
            .get("color")
            .and_then(|value| parse_hex_color(value))
            .unwrap_or(Color::new(0, 0, 0)),
        style,
        join: LineJoin::Miter,
        cap: LineCap::Flat,
    })
}

/// Scan current body sections in source order, ignoring historical section properties.
pub(in super::super) fn scan_page_borders(xml: &str) -> Vec<Option<PageBorders>> {
    let mut reader = NsReader::from_str(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut section_depth: Option<usize> = None;
    let mut border_depth: Option<usize> = None;
    let mut current: Option<PageBorders> = None;
    let mut result: Vec<Option<PageBorders>> = Vec::new();
    loop {
        match reader.read_event() {
            Ok(event @ (Event::Start(_) | Event::Empty(_))) => {
                let opens: bool = matches!(event, Event::Start(_));
                let (Event::Start(element) | Event::Empty(element)) = event else {
                    unreachable!()
                };
                let (namespace, name) = reader.resolver().resolve_element(element.name());
                let word: bool = is_word(namespace);
                let local = String::from_utf8_lossy(name.as_ref()).into_owned();
                let depth: usize = stack.len();
                if word
                    && local == "sectPr"
                    && section_depth.is_none()
                    && stack
                        .last()
                        .is_some_and(|name| matches!(name.as_str(), "body" | "pPr"))
                    && !stack
                        .iter()
                        .any(|name| matches!(name.as_str(), "sectPrChange" | "tc" | "txbxContent"))
                {
                    if opens {
                        section_depth = Some(depth);
                        current = None;
                    } else {
                        result.push(None);
                    }
                } else if word
                    && local == "pgBorders"
                    && section_depth.is_some()
                    && section_depth == depth.checked_sub(1)
                {
                    let values = attributes(&reader, &element);
                    current = Some(PageBorders {
                        offset_from_page: values
                            .get("offsetFrom")
                            .is_some_and(|value| value == "page"),
                        behind_text: values.get("zOrder").is_some_and(|value| value == "back"),
                        display: match values.get("display").map(String::as_str) {
                            Some("firstPage") => PageBorderDisplay::FirstPage,
                            Some("notFirstPage") => PageBorderDisplay::NotFirstPage,
                            _ => PageBorderDisplay::AllPages,
                        },
                        ..Default::default()
                    });
                    if opens {
                        border_depth = Some(depth);
                    }
                } else if word && border_depth.is_some() && border_depth == depth.checked_sub(1) {
                    let values = attributes(&reader, &element);
                    if let Some(borders) = current.as_mut() {
                        let parsed = side(&values);
                        let space: f64 = values
                            .get("space")
                            .and_then(|value| value.parse::<f64>().ok())
                            .filter(|value| value.is_finite() && *value >= 0.0)
                            .unwrap_or(0.0);
                        match local.as_str() {
                            "top" => {
                                borders.edges.top = parsed;
                                borders.spacing.top = space;
                            }
                            "bottom" => {
                                borders.edges.bottom = parsed;
                                borders.spacing.bottom = space;
                            }
                            "left" => {
                                borders.edges.left = parsed;
                                borders.spacing.left = space;
                            }
                            "right" => {
                                borders.edges.right = parsed;
                                borders.spacing.right = space;
                            }
                            _ => {}
                        }
                    }
                }
                if opens {
                    stack.push(if word { local } else { String::new() });
                }
            }
            Ok(Event::End(_)) => {
                stack.pop();
                let depth = stack.len();
                if border_depth == Some(depth) {
                    border_depth = None;
                }
                if section_depth == Some(depth) {
                    result.push(current.take());
                    section_depth = None;
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    result
}
