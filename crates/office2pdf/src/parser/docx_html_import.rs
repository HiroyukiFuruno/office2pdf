//! WordprocessingML bridge for the text/table subset of Word's HTML import.
//! docxide-pdf informed the CSS-to-paragraph mapping; Servo supplies HTML parsing.
use html5ever::{parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::{collections::BTreeMap, fmt::Write};

const WORD: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
type Properties = BTreeMap<String, String>;
struct Rule {
    selector: String,
    properties: Properties,
    specificity: usize,
}

fn tag(node: &Handle) -> &str {
    if let NodeData::Element { name, .. } = &node.data {
        name.local.as_ref()
    } else {
        ""
    }
}
fn attr(node: &Handle, key: &str) -> Option<String> {
    if let NodeData::Element { attrs, .. } = &node.data {
        attrs
            .borrow()
            .iter()
            .find(|a| a.name.local.as_ref() == key)
            .map(|a| a.value.to_string())
    } else {
        None
    }
}
fn escape(value: &str) -> String {
    quick_xml::escape::escape(value).into_owned()
}
fn declarations(value: &str) -> Properties {
    let mut properties = Properties::new();
    for declaration in value.split(';') {
        if let Some((key, value)) = declaration.split_once(':') {
            properties.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    properties
}
fn text(node: &Handle, depth: usize) -> String {
    if depth > 256 {
        return String::new();
    }
    if let NodeData::Text { contents } = &node.data {
        return contents.borrow().to_string();
    }
    node.children
        .borrow()
        .iter()
        .map(|child| text(child, depth + 1))
        .collect()
}
fn css_rules(node: &Handle, rules: &mut Vec<Rule>, depth: usize) -> Result<(), String> {
    if depth > 256 {
        return Err("HTML nesting exceeds import limit".into());
    }
    if tag(node) == "style" {
        let css = text(node, 0);
        for block in css.split('}') {
            if let Some((selectors, properties)) = block.split_once('{') {
                for selector in selectors.split(',') {
                    let selector = selector.trim();
                    if selector.contains([' ', '>', ':', '[', '+', '~']) {
                        continue;
                    }
                    rules.push(Rule {
                        selector: selector.into(),
                        properties: declarations(properties),
                        specificity: selector.matches('#').count() * 100
                            + selector.matches('.').count() * 10
                            + usize::from(selector.starts_with(char::is_alphabetic)),
                    });
                }
            }
        }
    }
    for child in node.children.borrow().iter() {
        css_rules(child, rules, depth + 1)?;
    }
    Ok(())
}
fn matches(node: &Handle, selector: &str) -> bool {
    let split = selector.find(['.', '#']).unwrap_or(selector.len());
    if split > 0 && &selector[..split] != "*" && selector[..split] != *tag(node) {
        return false;
    }
    let classes = attr(node, "class").unwrap_or_default();
    let id = attr(node, "id").unwrap_or_default();
    let mut rest = &selector[split..];
    while !rest.is_empty() {
        let kind = rest.as_bytes()[0];
        let end = rest[1..].find(['.', '#']).map_or(rest.len(), |i| i + 1);
        let value = &rest[1..end];
        if (kind == b'.' && !classes.split_ascii_whitespace().any(|c| c == value))
            || (kind == b'#' && id != value)
        {
            return false;
        }
        rest = &rest[end..];
    }
    !selector.is_empty()
}
fn computed(node: &Handle, parent: &Properties, rules: &[Rule]) -> Properties {
    let mut result: Properties = parent
        .iter()
        .filter(|(key, _)| {
            matches!(
                key.as_str(),
                "font-family"
                    | "font-size"
                    | "font-weight"
                    | "font-style"
                    | "color"
                    | "text-align"
                    | "line-height"
                    | "white-space"
            )
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    match tag(node) {
        "b" | "strong" | "th" => {
            result.insert("font-weight".into(), "bold".into());
        }
        "i" | "em" => {
            result.insert("font-style".into(), "italic".into());
        }
        "u" => {
            result.insert("text-decoration".into(), "underline".into());
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            result.insert(
                "font-size".into(),
                format!(
                    "{}pt",
                    match tag(node) {
                        "h1" => 24,
                        "h2" => 18,
                        _ => 12,
                    }
                ),
            );
            result.insert("font-weight".into(), "bold".into());
        }
        _ => {}
    }
    let mut matching: Vec<&Rule> = rules
        .iter()
        .filter(|r| matches(node, &r.selector))
        .collect();
    matching.sort_by_key(|r| r.specificity);
    for rule in matching {
        result.extend(rule.properties.clone());
    }
    if let Some(style) = attr(node, "style") {
        result.extend(declarations(&style));
    }
    result
}
fn length(value: &str) -> Option<f64> {
    let value = value.trim();
    let (number, scale) = [
        ("pt", 1.0),
        ("px", 0.75),
        ("in", 72.0),
        ("cm", 72.0 / 2.54),
        ("mm", 72.0 / 25.4),
    ]
    .into_iter()
    .find_map(|(unit, scale)| value.strip_suffix(unit).map(|n| (n, scale)))
    .unwrap_or((value, 1.0));
    number
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite())
        .map(|n| n * scale)
}
fn prop_length(props: &Properties, key: &str) -> Option<f64> {
    props.get(key).and_then(|value| length(value))
}
fn margin(props: &Properties, side: usize) -> Option<f64> {
    let key = ["margin-top", "margin-right", "margin-bottom", "margin-left"][side];
    prop_length(props, key).or_else(|| {
        let values: Vec<f64> = props
            .get("margin")?
            .split_ascii_whitespace()
            .map(length)
            .collect::<Option<_>>()?;
        let index = match (values.len(), side) {
            (1, _) => 0,
            (2, 0 | 2) => 0,
            (2, _) => 1,
            (3, 3) => 1,
            (_, s) => s,
        };
        values.get(index).copied()
    })
}
fn color(value: &str) -> Option<String> {
    let value = value.trim();
    let value = match value.to_ascii_lowercase().as_str() {
        "black" => "000000",
        "white" => "FFFFFF",
        "red" => "FF0000",
        "blue" => "0000FF",
        "green" => "008000",
        _ => value.strip_prefix('#')?,
    };
    if value.len() == 6 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(value.into())
    } else if value.len() == 3 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(value.chars().flat_map(|c| [c, c]).collect())
    } else {
        None
    }
}
fn run_properties(props: &Properties) -> String {
    let mut out = String::from("<w:rPr>");
    let family = props
        .get("font-family")
        .and_then(|v| v.split(',').next())
        .unwrap_or("Times New Roman")
        .trim()
        .trim_matches(['\'', '"']);
    let _ = write!(
        out,
        "<w:rFonts w:ascii=\"{}\" w:hAnsi=\"{}\"/>",
        escape(family),
        escape(family)
    );
    let size = prop_length(props, "font-size")
        .filter(|n| *n > 0.0)
        .unwrap_or(12.0);
    let _ = write!(out, "<w:sz w:val=\"{}\"/>", (size * 2.0).round() as u32);
    if let Some(weight) = props.get("font-weight") {
        let bold = weight == "bold" || weight.parse::<u32>().is_ok_and(|n| n >= 700);
        let _ = write!(out, "<w:b w:val=\"{}\"/>", u8::from(bold));
    }
    if let Some(style) = props.get("font-style") {
        let _ = write!(
            out,
            "<w:i w:val=\"{}\"/>",
            u8::from(style == "italic" || style == "oblique")
        );
    }
    if let Some(decoration) = props.get("text-decoration") {
        let _ = write!(
            out,
            "<w:u w:val=\"{}\"/>",
            if decoration.contains("underline") {
                "single"
            } else {
                "none"
            }
        );
    }
    if let Some(value) = props.get("color").and_then(|v| color(v)) {
        let _ = write!(out, "<w:color w:val=\"{value}\"/>");
    }
    out.push_str("</w:rPr>");
    out
}
struct HtmlRun {
    text: String,
    properties: Properties,
    line_break: bool,
}
fn inline(
    node: &Handle,
    parent: &Properties,
    rules: &[Rule],
    runs: &mut Vec<HtmlRun>,
    after_space: &mut bool,
    depth: usize,
) -> Result<(), String> {
    if depth > 256 {
        return Err("HTML nesting exceeds import limit".into());
    }
    if let NodeData::Text { contents } = &node.data {
        let mut value = String::new();
        for ch in contents.borrow().chars() {
            if ch.is_ascii_whitespace() {
                if !*after_space {
                    value.push(' ');
                }
                *after_space = true;
            } else {
                value.push(ch);
                *after_space = false;
            }
        }
        if !value.is_empty() {
            runs.push(HtmlRun {
                text: value,
                properties: parent.clone(),
                line_break: false,
            });
        }
        return Ok(());
    }
    if matches!(tag(node), "script" | "style" | "head" | "template") {
        return Ok(());
    }
    if tag(node) == "img" {
        return Ok(());
    }
    if tag(node) == "br" {
        runs.push(HtmlRun {
            text: String::new(),
            properties: parent.clone(),
            line_break: true,
        });
        *after_space = true;
        return Ok(());
    }
    let props = computed(node, parent, rules);
    for child in node.children.borrow().iter() {
        inline(child, &props, rules, runs, after_space, depth + 1)?;
    }
    Ok(())
}
fn span_margins(
    nodes: &[Handle],
    parent: &Properties,
    rules: &[Rule],
    depth: usize,
) -> Option<(Option<f64>, Option<f64>)> {
    if depth > 256 {
        return None;
    }
    for node in nodes {
        let props = computed(node, parent, rules);
        if tag(node) == "span" {
            let pair = (margin(&props, 0), margin(&props, 2));
            if pair.0.is_some() || pair.1.is_some() {
                return Some(pair);
            }
        }
        if let Some(pair) = span_margins(&node.children.borrow(), &props, rules, depth + 1) {
            return Some(pair);
        }
    }
    None
}

fn paragraph_nodes(
    nodes: &[Handle],
    props: &Properties,
    rules: &[Rule],
    prefix: &str,
) -> Result<String, String> {
    let mut out = format!("<w:p xmlns:w=\"{WORD}\"><w:pPr>");
    if let Some(alignment) = props
        .get("text-align")
        .filter(|a| matches!(a.as_str(), "left" | "right" | "center" | "justify"))
    {
        let _ = write!(
            out,
            "<w:jc w:val=\"{}\"/>",
            if alignment == "justify" {
                "both"
            } else {
                alignment
            }
        );
    }
    out.push_str("<w:spacing");
    // Word's HTML importer applies span margins to explicitly spaced
    // paragraphs; omitted/invalid paragraph margins still use auto spacing.
    // This mapping follows docxide-pdf's native Word import probes.
    let spans = span_margins(nodes, props, rules, 0).unwrap_or_default();
    for (side, name) in [(0, "before"), (2, "after")] {
        if let Some(own) = margin(props, side) {
            let value = if side == 0 { spans.0 } else { spans.1 }.unwrap_or(own);
            let _ = write!(
                out,
                " w:{name}=\"{}\"",
                (value.max(0.0) * 20.0).round() as u32
            );
        } else {
            let _ = write!(out, " w:{name}=\"280\" w:{name}Autospacing=\"1\"");
        }
    }
    let line = props
        .get("line-height")
        .and_then(|value| {
            if let Some(pct) = value.strip_suffix('%') {
                Some((pct.parse::<f64>().ok()? * 2.4, "auto"))
            } else if let Ok(multiplier) = value.parse::<f64>() {
                Some((multiplier * 240.0, "auto"))
            } else {
                Some((length(value)? * 20.0, "exact"))
            }
        })
        .filter(|(n, _)| n.is_finite() && *n > 0.0)
        .unwrap_or((240.0, "auto"));
    let _ = write!(
        out,
        " w:line=\"{}\" w:lineRule=\"{}\"/>",
        line.0.round() as u32,
        line.1
    );
    if margin(props, 3).is_some() || prop_length(props, "text-indent").is_some() {
        let _ = write!(
            out,
            "<w:ind w:left=\"{}\" w:firstLine=\"{}\"/>",
            (margin(props, 3).unwrap_or(0.0) * 20.0).round() as i32,
            (prop_length(props, "text-indent").unwrap_or(0.0) * 20.0).round() as i32
        );
    }
    out.push_str("<w:widowControl/></w:pPr>");
    let mut runs = Vec::new();
    let mut after_space = true;
    if !prefix.is_empty() {
        runs.push(HtmlRun {
            text: prefix.into(),
            properties: props.clone(),
            line_break: false,
        });
    }
    for child in nodes {
        inline(child, props, rules, &mut runs, &mut after_space, 0)?;
    }
    if let Some(last) = runs.iter_mut().rev().find(|run| !run.text.is_empty())
        && last.text.ends_with(' ')
    {
        last.text.pop();
    }
    for run in runs {
        let _ = write!(out, "<w:r>{}", run_properties(&run.properties));
        if run.line_break {
            out.push_str("<w:br/>");
        } else {
            let _ = write!(
                out,
                "<w:t xml:space=\"preserve\">{}</w:t>",
                escape(&run.text)
            );
        }
        out.push_str("</w:r>");
    }
    out.push_str("</w:p>");
    Ok(out)
}
fn paragraph(
    node: &Handle,
    props: &Properties,
    rules: &[Rule],
    prefix: &str,
) -> Result<String, String> {
    paragraph_nodes(&node.children.borrow(), props, rules, prefix)
}
fn table_rows(node: &Handle, rows: &mut Vec<Handle>, depth: usize) -> Result<(), String> {
    if depth > 256 {
        return Err("HTML nesting exceeds import limit".into());
    }
    for child in node.children.borrow().iter() {
        if tag(child) == "tr" {
            rows.push(child.clone());
        } else if matches!(tag(child), "thead" | "tbody" | "tfoot") {
            table_rows(child, rows, depth + 1)?;
        }
    }
    Ok(())
}
fn table(
    node: &Handle,
    props: &Properties,
    rules: &[Rule],
    depth: usize,
) -> Result<String, String> {
    let mut rows = Vec::new();
    table_rows(node, &mut rows, 0)?;
    let mut out =
        format!("<w:tbl xmlns:w=\"{WORD}\"><w:tblPr><w:tblW w:type=\"auto\" w:w=\"0\"/></w:tblPr>");
    for row in rows {
        let row_props = computed(&row, props, rules);
        out.push_str("<w:tr>");
        for cell in row
            .children
            .borrow()
            .iter()
            .filter(|cell| matches!(tag(cell), "td" | "th"))
        {
            let cell_props = computed(cell, &row_props, rules);
            out.push_str("<w:tc><w:tcPr>");
            if let Some(span) = attr(cell, "colspan")
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|v| *v > 1 && *v <= 1024)
            {
                let _ = write!(out, "<w:gridSpan w:val=\"{span}\"/>");
            }
            if let Some(width) = prop_length(&cell_props, "width").filter(|v| *v > 0.0) {
                let _ = write!(
                    out,
                    "<w:tcW w:w=\"{}\" w:type=\"dxa\"/>",
                    (width * 20.0).round() as u32
                );
            }
            out.push_str("</w:tcPr>");
            let blocks = children_blocks(cell, &cell_props, rules, depth + 1)?;
            if blocks.is_empty() {
                out.push_str(&paragraph(cell, &cell_props, rules, "")?);
            } else {
                out.push_str(&blocks);
            }
            out.push_str("</w:tc>");
        }
        out.push_str("</w:tr>");
    }
    out.push_str("</w:tbl>");
    Ok(out)
}
fn flush_inline(
    nodes: &mut Vec<Handle>,
    props: &Properties,
    rules: &[Rule],
    out: &mut String,
) -> Result<(), String> {
    if nodes.is_empty() {
        return Ok(());
    }
    if nodes
        .iter()
        .any(|node| text(node, 0).chars().any(|ch| !ch.is_ascii_whitespace()))
    {
        out.push_str(&paragraph_nodes(nodes, props, rules, "")?);
    }
    nodes.clear();
    Ok(())
}

fn children_blocks(
    node: &Handle,
    props: &Properties,
    rules: &[Rule],
    depth: usize,
) -> Result<String, String> {
    if depth > 256 {
        return Err("HTML nesting exceeds import limit".into());
    }
    let mut out = String::new();
    let mut pending = Vec::new();
    for child in node.children.borrow().iter() {
        let child_props = computed(child, props, rules);
        let name = tag(child);
        if matches!(name, "head" | "script" | "style" | "template") {
            continue;
        }
        if !matches!(
            name,
            "p" | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "pre"
                | "blockquote"
                | "table"
                | "ul"
                | "ol"
                | "html"
                | "body"
                | "div"
                | "section"
                | "article"
                | "main"
        ) {
            pending.push(child.clone());
            continue;
        }
        flush_inline(&mut pending, props, rules, &mut out)?;
        match name {
            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "pre" | "blockquote" => {
                out.push_str(&paragraph(child, &child_props, rules, "")?)
            }
            "table" => out.push_str(&table(child, &child_props, rules, depth + 1)?),
            "ul" | "ol" => {
                let mut number = attr(child, "start")
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(1);
                for item in child
                    .children
                    .borrow()
                    .iter()
                    .filter(|item| tag(item) == "li")
                {
                    let item_props = computed(item, &child_props, rules);
                    out.push_str(&paragraph(
                        item,
                        &item_props,
                        rules,
                        &if name == "ol" {
                            format!("{number}. ")
                        } else {
                            "• ".into()
                        },
                    )?);
                    number = number.saturating_add(1);
                }
            }
            _ => out.push_str(&children_blocks(child, &child_props, rules, depth + 1)?),
        }
    }
    flush_inline(&mut pending, props, rules, &mut out)?;
    Ok(out)
}

fn unsupported(node: &Handle, findings: &mut std::collections::BTreeSet<String>, depth: usize) {
    if depth > 256 {
        return;
    }
    if tag(node) == "img" {
        findings.insert(
            "HTML images are not supported in altChunk imports; surrounding text is retained"
                .into(),
        );
    }
    if attr(node, "rowspan").is_some_and(|v| v != "1") {
        findings.insert(
            "HTML rowspan is not supported in altChunk imports; cell text is retained".into(),
        );
    }
    for child in node.children.borrow().iter() {
        unsupported(child, findings, depth + 1);
    }
}

pub(super) fn html_to_wordml(html: &str) -> Result<(String, Vec<String>), String> {
    let dom = parse_document(RcDom::default(), Default::default()).one(html);
    let mut rules = Vec::new();
    css_rules(&dom.document, &mut rules, 0)?;
    let mut findings = std::collections::BTreeSet::new();
    unsupported(&dom.document, &mut findings, 0);
    Ok((
        children_blocks(&dom.document, &Properties::new(), &rules, 0)?,
        findings.into_iter().collect(),
    ))
}
