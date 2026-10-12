use std::collections::HashMap;

use super::page_borders::{attributes, is_word};
use quick_xml::{events::Event, reader::NsReader};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum NumberingDefinition {
    Abstract(usize),
    Instance(usize),
}

/// The pinned reader ignores OnOff values and namespaces for isLgl. Read
/// namespace-qualified flags directly, including complete level overrides.
pub(in crate::parser) fn normalize_legal_numbering(
    xml: &str,
    numberings: &mut docx_rs::Numberings,
) {
    let mut reader = NsReader::from_str(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut definition: Option<(NumberingDefinition, usize)> = None;
    let mut level: Option<(usize, usize)> = None;
    let mut flags: HashMap<(NumberingDefinition, usize), bool> = HashMap::new();
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
                if word && stack == ["numbering"] && matches!(local.as_str(), "abstractNum" | "num")
                {
                    let values = attributes(&reader, &element);
                    let key = if local == "abstractNum" {
                        "abstractNumId"
                    } else {
                        "numId"
                    };
                    definition = values.get(key).and_then(|id| id.parse().ok()).map(|id| {
                        (
                            if local == "abstractNum" {
                                NumberingDefinition::Abstract(id)
                            } else {
                                NumberingDefinition::Instance(id)
                            },
                            depth,
                        )
                    });
                } else if word
                    && local == "lvl"
                    && definition.is_some()
                    && (stack == ["numbering", "abstractNum"]
                        || stack == ["numbering", "num", "lvlOverride"])
                {
                    level = attributes(&reader, &element)
                        .get("ilvl")
                        .and_then(|index| index.parse().ok())
                        .map(|index| (index, depth));
                } else if word
                    && local == "isLgl"
                    && level.is_some_and(|(_, level_depth)| level_depth + 1 == depth)
                {
                    let values = attributes(&reader, &element);
                    let enabled = matches!(
                        values.get("val").map(String::as_str),
                        None | Some("true" | "1" | "on")
                    );
                    if let (Some((definition, _)), Some((level, _))) = (definition, level) {
                        flags.insert((definition, level), enabled);
                    }
                }
                if opens {
                    stack.push(if word { local } else { String::new() });
                } else {
                    if level.is_some_and(|(_, level_depth)| level_depth == depth) {
                        level = None;
                    }
                    if definition.is_some_and(|(_, definition_depth)| definition_depth == depth) {
                        definition = None;
                    }
                }
            }
            Ok(Event::End(_)) => {
                stack.pop();
                if level.is_some_and(|(_, depth)| depth == stack.len()) {
                    level = None;
                }
                if definition.is_some_and(|(_, depth)| depth == stack.len()) {
                    definition = None;
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    for abstract_numbering in &mut numberings.abstract_nums {
        for level in &mut abstract_numbering.levels {
            level.is_lgl = flags
                .get(&(
                    NumberingDefinition::Abstract(abstract_numbering.id),
                    level.level,
                ))
                .copied()
                .unwrap_or(false)
                .then(docx_rs::IsLgl::new);
        }
    }
    for numbering in &mut numberings.numberings {
        for override_level in &mut numbering.level_overrides {
            if let Some(level) = &mut override_level.override_level {
                level.is_lgl = flags
                    .get(&(NumberingDefinition::Instance(numbering.id), level.level))
                    .copied()
                    .unwrap_or(false)
                    .then(docx_rs::IsLgl::new);
            }
        }
    }
}
