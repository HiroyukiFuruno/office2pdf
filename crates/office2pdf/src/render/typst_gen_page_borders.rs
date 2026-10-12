use super::{fmt, format_f64, section_first_page_expression};
use crate::ir::{BorderLineStyle, BorderSide, FlowPage, PageBorderDisplay, PageSize};
use std::fmt::Write;

fn extent(side: &BorderSide) -> f64 {
    if side.style == BorderLineStyle::Double {
        side.width * 3.0
    } else {
        side.width
    }
}

fn edge_offset(side: &BorderSide, spacing: f64, margin: f64, from_page: bool) -> f64 {
    let inset = spacing + extent(side) / 2.0;
    if from_page { inset } else { margin - inset }
}

fn draw_edge(out: &mut String, side: &BorderSide, x: f64, y: f64, length: f64, vertical: bool) {
    if length <= 0.0 || !length.is_finite() {
        return;
    }
    let shifts: &[f64] = if side.style == BorderLineStyle::Double {
        &[-1.0, 1.0]
    } else {
        &[0.0]
    };
    for shift in shifts {
        let offset = shift * side.width;
        let (x, y) = if vertical {
            (x + offset, y)
        } else {
            (x, y + offset)
        };
        let mut stroke = side.clone();
        if stroke.style == BorderLineStyle::Double {
            stroke.style = BorderLineStyle::Solid;
        }
        let _ = write!(
            out,
            "#place(top + left, dx: {}pt, dy: {}pt, line(length: {}pt, angle: {}deg, stroke: {}))",
            format_f64(x),
            format_f64(y),
            format_f64(length),
            if vertical { 90 } else { 0 },
            fmt::stroke_value(&stroke, true)
        );
    }
}

/// Page-space geometry follows docxide-pdf's assembly model and native Word probes.
/// Border space measures to the outer ink edge, not the centered line path.
pub(super) fn page_border_layer_markup(
    page: &FlowPage,
    size: &PageSize,
    behind_text: bool,
    section: usize,
) -> String {
    let Some(borders) = page
        .page_borders
        .as_ref()
        .filter(|border| border.behind_text == behind_text)
    else {
        return String::new();
    };
    let edge = |side: &Option<BorderSide>, spacing: f64, margin: f64, far: bool, total: f64| {
        side.as_ref().map(|side| {
            let inset = edge_offset(side, spacing, margin, borders.offset_from_page);
            if far { total - inset } else { inset }
        })
    };
    // An absent edge still bounds adjacent edges at the page/text boundary.
    let fallback = |margin: f64| {
        if borders.offset_from_page {
            0.0
        } else {
            margin
        }
    };
    let left = edge(
        &borders.edges.left,
        borders.spacing.left,
        page.margins.left,
        false,
        size.width,
    )
    .unwrap_or(fallback(page.margins.left));
    let right = edge(
        &borders.edges.right,
        borders.spacing.right,
        page.margins.right,
        true,
        size.width,
    )
    .unwrap_or(size.width - fallback(page.margins.right));
    let top = edge(
        &borders.edges.top,
        borders.spacing.top,
        page.margins.top,
        false,
        size.height,
    )
    .unwrap_or(fallback(page.margins.top));
    let bottom = edge(
        &borders.edges.bottom,
        borders.spacing.bottom,
        page.margins.bottom,
        true,
        size.height,
    )
    .unwrap_or(size.height - fallback(page.margins.bottom));
    let left_extension = borders.edges.left.as_ref().map(extent).unwrap_or(0.0) / 2.0;
    let right_extension = borders.edges.right.as_ref().map(extent).unwrap_or(0.0) / 2.0;
    let mut output = String::new();
    for (side, y) in [(&borders.edges.top, top), (&borders.edges.bottom, bottom)] {
        if let Some(side) = side {
            draw_edge(
                &mut output,
                side,
                left - left_extension,
                y,
                right - left + left_extension + right_extension,
                false,
            );
        }
    }
    for (side, x) in [(&borders.edges.left, left), (&borders.edges.right, right)] {
        if let Some(side) = side {
            draw_edge(&mut output, side, x, top, bottom - top, true);
        }
    }
    if output.is_empty() {
        return output;
    }
    match borders.display {
        PageBorderDisplay::AllPages => output,
        PageBorderDisplay::FirstPage | PageBorderDisplay::NotFirstPage => format!(
            "#context {{ if here().page() {} {} [{}] }}",
            if borders.display == PageBorderDisplay::FirstPage {
                "=="
            } else {
                "!="
            },
            section_first_page_expression(section),
            output
        ),
    }
}
