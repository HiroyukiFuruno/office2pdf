use super::page_borders::{attributes, is_word};
use crate::ir::PageVerticalAlignment;
use quick_xml::{events::Event, reader::NsReader};

pub(in crate::parser) fn scan_section_alignment(xml: &str) -> Vec<Option<PageVerticalAlignment>> {
    let mut reader = NsReader::from_str(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut section_depth: Option<usize> = None;
    let mut current = None;
    let mut result = Vec::new();
    loop {
        match reader.read_event() {
            Ok(event @ (Event::Start(_) | Event::Empty(_))) => {
                let opens = matches!(event, Event::Start(_));
                let (Event::Start(element) | Event::Empty(element)) = event else {
                    unreachable!()
                };
                let (namespace, name) = reader.resolver().resolve_element(element.name());
                let word = is_word(namespace);
                let local = String::from_utf8_lossy(name.as_ref()).into_owned();
                let depth = stack.len();
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
                    && local == "vAlign"
                    && section_depth == depth.checked_sub(1)
                    && section_depth.is_some()
                {
                    let values = attributes(&reader, &element);
                    current = match values.get("val").map(String::as_str) {
                        Some("top") => Some(PageVerticalAlignment::Top),
                        Some("center") => Some(PageVerticalAlignment::Center),
                        Some("bottom") => Some(PageVerticalAlignment::Bottom),
                        Some("both") => Some(PageVerticalAlignment::Justified),
                        _ => None,
                    };
                }
                if opens {
                    stack.push(if word { local } else { String::new() });
                }
            }
            Ok(Event::End(_)) => {
                stack.pop();
                if section_depth == Some(stack.len()) {
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
