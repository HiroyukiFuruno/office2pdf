//! Expand otherwise unsupported cached diagram presets through our existing
//! custom-geometry evaluator. Native preset implementations remain authoritative.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::OnceLock;

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::{ConvertError, ConvertWarning};
use crate::ir::{ArrowHead, ShapeKind};
use crate::parser::xml_util::get_attr_str;

const DEFINITIONS: &str = include_str!("drawingml_presets/presetShapeDefinitions.xml");

fn definitions() -> &'static HashMap<String, Range<usize>> {
    static INDEX: OnceLock<HashMap<String, Range<usize>>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut reader = Reader::from_str(DEFINITIONS);
        let mut result = HashMap::new();
        let mut depth: usize = 0;
        let mut current: Option<(String, usize)> = None;
        loop {
            let start: usize = reader.buffer_position() as usize;
            match reader.read_event() {
                Ok(Event::Start(element)) => {
                    if depth == 1 {
                        current = Some((
                            String::from_utf8_lossy(element.name().as_ref()).into_owned(),
                            reader.buffer_position() as usize,
                        ));
                    }
                    depth += 1;
                }
                Ok(Event::End(_)) => {
                    if depth == 2
                        && let Some((name, content_start)) = current.take()
                    {
                        result.insert(name, content_start..start);
                    }
                    depth = depth.saturating_sub(1);
                }
                Ok(Event::Eof) | Err(_) => break,
                _ => {}
            }
        }
        result
    })
}

fn lacks_native_outline(preset: &str) -> bool {
    preset != "rect"
        && matches!(
            super::shapes::prst_to_shape_kind(
                preset,
                1.0,
                1.0,
                false,
                false,
                ArrowHead::None,
                ArrowHead::None,
                &[],
            ),
            ShapeKind::Rectangle
        )
}

fn definition_guide(
    element: &BytesStart<'_>,
    adjustments: Option<&HashMap<String, String>>,
) -> BytesStart<'static> {
    let mut updated = element.to_owned();
    let override_formula = get_attr_str(element, b"name")
        .and_then(|name| adjustments.and_then(|values| values.get(&name).cloned()));
    let padded_definition = get_attr_str(element, b"fmla").and_then(|formula| {
        let operands: Vec<&str> = formula.split_whitespace().collect();
        // Eight Annex D guides pad the three-operand +- formula with a
        // fourth zero. docxide's generator also uses only the first three.
        (operands.len() == 5 && operands[0] == "+-" && operands[4] == "0")
            .then(|| operands[..4].join(" "))
    });
    if let Some(formula) = override_formula.or(padded_definition) {
        updated.clear_attributes();
        for attribute in element.attributes().flatten() {
            if attribute.key.as_ref() != b"fmla" {
                updated.push_attribute(attribute);
            }
        }
        let escaped = quick_xml::escape::escape(&formula);
        updated.push_attribute(("fmla", escaped.as_ref()));
    }
    updated
}

fn geometry_xml(
    preset: &str,
    qualified_name: &str,
    adjustments: &HashMap<String, String>,
) -> Result<Option<Vec<u8>>, ConvertError> {
    let Some(span) = definitions().get(preset) else {
        return Ok(None);
    };
    let mut reader = Reader::from_str(&DEFINITIONS[span.clone()]);
    let mut writer = Writer::new(Vec::new());
    let mut depth: usize = 0;
    let mut selected: bool = false;
    let mut in_adjustments: bool = false;
    let custom_name: String = qualified_name.rsplit_once(':').map_or_else(
        || "custGeom".to_string(),
        |(prefix, _)| format!("{prefix}:custGeom"),
    );
    writer.write_event(Event::Start(BytesStart::new(&custom_name)))?;
    loop {
        let event = reader
            .read_event()
            .map_err(|error| ConvertError::Parse(error.to_string()))?;
        match &event {
            Event::Start(element) | Event::Empty(element) => {
                let local = element.local_name();
                if depth == 0 {
                    selected = matches!(local.as_ref(), b"avLst" | b"gdLst" | b"pathLst");
                    in_adjustments = local.as_ref() == b"avLst";
                }
                // The shared IR gives a shape one fill and one stroke. Keep
                // compound presets with different per-path paint explicit
                // rather than silently flattening their shading or line art.
                if selected
                    && local.as_ref() == b"path"
                    && (get_attr_str(element, b"fill").is_some_and(|fill| fill != "norm")
                        || get_attr_str(element, b"stroke")
                            .is_some_and(|stroke| stroke != "1" && stroke != "true"))
                {
                    return Ok(None);
                }
                if selected {
                    if local.as_ref() == b"gd" {
                        let updated =
                            definition_guide(element, in_adjustments.then_some(adjustments));
                        writer.write_event(if matches!(event, Event::Start(_)) {
                            Event::Start(updated)
                        } else {
                            Event::Empty(updated)
                        })?;
                    } else {
                        writer.write_event(event.clone())?;
                    }
                }
                if matches!(event, Event::Start(_)) {
                    depth += 1;
                }
            }
            Event::End(_) => {
                if selected {
                    writer.write_event(event.clone())?;
                }
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    selected = false;
                }
            }
            Event::Eof => break,
            _ => {
                if selected {
                    writer.write_event(event)?;
                }
            }
        }
    }
    // Cached SmartArt supplies a separate txXfrm text rectangle; copying
    // the preset's a:rect as well would inset that saved rectangle twice.
    writer.write_event(Event::End(BytesEnd::new(&custom_name)))?;
    Ok(Some(writer.into_inner()))
}

pub(super) fn expand_cached_presets(
    xml: &str,
) -> Result<(String, Vec<ConvertWarning>), ConvertError> {
    let mut reader = Reader::from_str(xml);
    let mut output = Writer::new(Vec::new());
    let mut warnings = Vec::new();
    loop {
        let event = reader
            .read_event()
            .map_err(|error| ConvertError::Parse(error.to_string()))?;
        match &event {
            Event::Start(element) | Event::Empty(element)
                if element.local_name().as_ref() == b"prstGeom" =>
            {
                let preset = get_attr_str(element, b"prst").unwrap_or_default();
                if !lacks_native_outline(&preset) {
                    output.write_event(event)?;
                    continue;
                }
                let qualified_name = String::from_utf8_lossy(element.name().as_ref()).into_owned();
                let mut original = Writer::new(Vec::new());
                original.write_event(event.clone())?;
                let mut adjustments = HashMap::new();
                if matches!(event, Event::Start(_)) {
                    let mut depth: usize = 1;
                    while depth > 0 {
                        let child = reader
                            .read_event()
                            .map_err(|error| ConvertError::Parse(error.to_string()))?;
                        match &child {
                            Event::Start(child) | Event::Empty(child) => {
                                if child.local_name().as_ref() == b"gd"
                                    && let (Some(name), Some(formula)) =
                                        (get_attr_str(child, b"name"), get_attr_str(child, b"fmla"))
                                {
                                    adjustments.insert(name, formula);
                                }
                            }
                            Event::End(_) => depth -= 1,
                            Event::Eof => {
                                return Err(ConvertError::Parse(
                                    "Unclosed cached preset geometry".into(),
                                ));
                            }
                            _ => {}
                        }
                        if matches!(child, Event::Start(_)) {
                            depth += 1;
                        }
                        original.write_event(child)?;
                    }
                }
                if let Some(geometry) = geometry_xml(&preset, &qualified_name, &adjustments)? {
                    output.get_mut().extend(geometry);
                } else {
                    output.get_mut().extend(original.into_inner());
                    warnings.push(ConvertWarning::PartialElement {
                        format: "DrawingML".into(),
                        element: format!("cached preset {preset}"),
                        detail: "Preset outline or per-path painting is unsupported; using bounding rectangle".into(),
                    });
                }
            }
            Event::Eof => break,
            _ => output.write_event(event)?,
        }
    }
    String::from_utf8(output.into_inner())
        .map(|xml| (xml, warnings))
        .map_err(|error| ConvertError::Parse(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::FixedElementKind;

    fn diagram(preset: &str, adjustments: &str, width: i64, height: i64) -> String {
        format!(
            r#"<dsp:drawing xmlns:dsp="http://schemas.microsoft.com/office/drawing/2008/diagram" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><dsp:sp><dsp:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{width}" cy="{height}"/></a:xfrm><a:prstGeom prst="{preset}"><a:avLst>{adjustments}</a:avLst></a:prstGeom><a:solidFill><a:srgbClr val="176886"/></a:solidFill></dsp:spPr></dsp:sp></dsp:drawing>"#,
        )
    }

    #[test]
    fn cached_diagram_arrows_and_gears_keep_finite_nonrectangular_paths() {
        for preset in [
            "quadArrow",
            "leftRightArrow",
            "gear6",
            "gear9",
            "circularArrow",
            "leftCircularArrow",
        ] {
            for (width, height) in [(7_620_000, 3_810_000), (3_810_000, 7_620_000)] {
                let (elements, warnings) = super::super::slides::parse_cached_drawing(
                    &diagram(preset, "", width, height),
                    &super::super::ThemeData::default(),
                    &super::super::default_color_map(),
                    &super::super::SlideImageMap::new(),
                )
                .unwrap();
                assert!(warnings.is_empty(), "{preset}: {warnings:?}");
                let FixedElementKind::Shape(shape) = &elements[0].kind else {
                    panic!("shape {preset}")
                };
                let ShapeKind::Path { subpaths } = &shape.kind else {
                    panic!("outline {preset}")
                };
                assert!(!subpaths.is_empty(), "{preset}");
                assert!(
                    subpaths.iter().any(|path| path.vertices.len() > 4),
                    "{preset}"
                );
                assert!(
                    subpaths
                        .iter()
                        .flat_map(|path| &path.vertices)
                        .all(|(x, y)| x.is_finite() && y.is_finite()),
                    "{preset}"
                );
            }
        }
    }

    #[test]
    fn cached_preset_adjustments_are_matched_by_name() {
        let ordered = r#"<a:gd name="adj1" fmla="val 2000"/><a:gd name="adj2" fmla="val 4000"/><a:gd name="adj3" fmla="val 5000"/>"#;
        let reordered = r#"<a:gd name="adj3" fmla="val 5000"/><a:gd name="adj1" fmla="val 2000"/><a:gd name="adj2" fmla="val 4000"/>"#;
        let (first, _) =
            expand_cached_presets(&diagram("quadArrow", ordered, 7_620_000, 3_810_000)).unwrap();
        let (second, _) =
            expand_cached_presets(&diagram("quadArrow", reordered, 7_620_000, 3_810_000)).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn cached_presets_keep_native_outlines_and_warn_for_unrepresentable_paint() {
        for preset in ["rect", "ellipse", "roundRect", "pie"] {
            let xml = diagram(preset, "", 7_620_000, 3_810_000);
            let (expanded, warnings) = expand_cached_presets(&xml).unwrap();
            assert_eq!(expanded, xml);
            assert!(warnings.is_empty());
        }
        for preset in ["cube", "unrecognizedPreset"] {
            let xml = diagram(preset, "", 7_620_000, 3_810_000);
            let (expanded, warnings) = expand_cached_presets(&xml).unwrap();
            assert_eq!(expanded, xml);
            assert_eq!(warnings.len(), 1);
        }
    }
}
