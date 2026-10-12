use super::{GenCtx, format_f64};
use crate::ir::{FlowPage, LineNumberRestart};
use std::fmt::Write;

pub(super) fn write_line_numbering(out: &mut String, page: &FlowPage, ctx: &mut GenCtx) {
    let Some(numbering) = &page.line_numbering else {
        if ctx.has_flow_line_numbering {
            out.push_str("#set par.line(numbering: none)\n");
            ctx.has_flow_line_numbering = false;
        }
        return;
    };
    ctx.has_flow_line_numbering = true;
    let mut body = String::new();
    match numbering.restart {
        LineNumberRestart::Continuous => {
            // Word increments before displaying a continuously numbered line.
            let _ = write!(body, "let value = n + {};", numbering.start);
        }
        LineNumberRestart::NewPage | LineNumberRestart::NewSection => {
            // Typst's line counter cannot be reset directly. A located first
            // value provides Word's page/section base while its scope stays
            // uniform even across continuous section boundaries.
            let key = if numbering.restart == LineNumberRestart::NewPage {
                "str(here().page())"
            } else {
                "\"section\""
            };
            let _ = write!(
                body,
                "let store = state(\"o2p-line-base-{}\", (:)); let bases = store.get(); let key = {key}; let base = bases.at(key, default: n); if not bases.keys().contains(key) {{ bases.insert(key, n); store.update(bases) }}; let value = n - base + {};",
                ctx.flow_section_index, numbering.start
            );
        }
    }
    // Margin numbers are outside the styled body runs. Without explicit
    // document defaults they would use Typst's own Libertinus face.
    let mut text_parameters = String::new();
    if let Some(style) = ctx.document_default_text.as_ref() {
        let mut style = style.clone();
        style.color = None;
        super::text::write_text_params(&mut text_parameters, &style);
    }
    if !text_parameters.is_empty() {
        text_parameters.push_str(", ");
    }
    let _ = writeln!(
        out,
        "#set par.line(numbering: n => context {{ {body} if value >= 1 and calc.rem(value, {}) == 0 {{ text({text_parameters}fill: black, str(value)) }} }}, number-align: right, number-margin: left, number-clearance: {}pt, numbering-scope: \"document\")",
        numbering.count_by.max(1),
        format_f64(numbering.distance.unwrap_or(18.0))
    );
    out.push_str("#show table: set par.line(numbering: none)\n");
}
