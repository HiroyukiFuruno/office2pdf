#![cfg(not(target_arch = "wasm32"))] // native-only integration tests (fs, qpdf, criterion)
//! Integration tests for PPTX fixtures.
//!
//! Each real-world `.pptx` file in `tests/fixtures/pptx/` gets two tests:
//! - **smoke**: `convert()` → valid PDF (or graceful error — no panic)
//! - **structure**: parse → assert expected IR content

mod common;

use std::path::PathBuf;

use office2pdf::config::ConvertOptions;
use office2pdf::internal::Parser;
use office2pdf::internal::PptxParser;
use office2pdf::internal::generate_typst;
use office2pdf::ir::{Block, Color, FixedElementKind, FixedPage, LineSpacing, Page};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/pptx")
        .join(name)
}

fn load_fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name)).expect("fixture file should exist")
}

/// Smoke-test helper: conversion must not panic.
fn assert_produces_valid_pdf(name: &str) {
    let path = fixture_path(name);
    match office2pdf::convert(&path) {
        Ok(result) => {
            assert!(!result.pdf.is_empty(), "PDF output should not be empty");
            assert!(
                result.pdf.starts_with(b"%PDF"),
                "output should start with PDF magic bytes"
            );
            common::validate_pdf_with_qpdf(&result.pdf);
        }
        Err(e) => {
            eprintln!("[WARN] {name}: conversion error (non-panic): {e}");
        }
    }
}

/// Parse a PPTX fixture and return the fixed pages (slides).
fn fixed_pages(name: &str) -> Vec<FixedPage> {
    let data = load_fixture(name);
    let (doc, _warnings) = PptxParser.parse(&data, &ConvertOptions::default()).unwrap();
    doc.pages
        .into_iter()
        .filter_map(|p| match p {
            Page::Fixed(fp) => Some(fp),
            _ => None,
        })
        .collect()
}

fn has_fixed_image(pages: &[FixedPage]) -> bool {
    pages
        .iter()
        .flat_map(|p| p.elements.iter())
        .any(|e| matches!(e.kind, FixedElementKind::Image(_)))
}

fn has_textbox_with_content(pages: &[FixedPage]) -> bool {
    pages
        .iter()
        .flat_map(|p| p.elements.iter())
        .any(|e| match &e.kind {
            FixedElementKind::TextBox(text_box) => text_box.content.iter().any(|b| match b {
                Block::Paragraph(para) => para.runs.iter().any(|r| !r.text.is_empty()),
                _ => false,
            }),
            _ => false,
        })
}

// ---------------------------------------------------------------------------
// PR #188 contributor acceptance fixtures
// ---------------------------------------------------------------------------

const PR_188_PAGE_FILL_FIXTURE: &str = "pr_188_page_fill_reset.pptx";
const PR_188_LAYOUT_GRADIENT_FIXTURE: &str = "pr_188_layout_gradient.pptx";
const PR_188_MASTER_BG_REF_FIXTURE: &str = "pr_188_master_bg_ref.pptx";

#[test]
fn structure_pr_188_contributor_acceptance_supported_behavior() {
    let reset_pages = fixed_pages(PR_188_PAGE_FILL_FIXTURE);
    assert_eq!(reset_pages.len(), 2);
    assert_eq!(
        reset_pages[0].background_color,
        Some(Color::new(0xC0, 0x00, 0x00))
    );
    assert_eq!(reset_pages[1].background_color, None);
    assert!(reset_pages[1].background_gradient.is_none());

    let gradient_pages = fixed_pages(PR_188_LAYOUT_GRADIENT_FIXTURE);
    let gradient = gradient_pages[0]
        .background_gradient
        .as_ref()
        .expect("slide should inherit the layout gradient");
    assert_eq!(gradient.stops.len(), 2);
    assert_eq!(gradient.stops[0].color, Color::new(0x11, 0x22, 0x33));
    assert_eq!(gradient.stops[1].color, Color::new(0x44, 0x55, 0x66));

    let bg_ref_pages = fixed_pages(PR_188_MASTER_BG_REF_FIXTURE);
    assert_eq!(
        bg_ref_pages[0].background_color,
        Some(Color::new(0x44, 0x72, 0xC4)),
        "the master's bgRef should resolve the first theme background fill with accent1"
    );
}

#[test]
fn smoke_pr_188_contributor_acceptance_fixtures() {
    for fixture in [
        PR_188_PAGE_FILL_FIXTURE,
        PR_188_LAYOUT_GRADIENT_FIXTURE,
        PR_188_MASTER_BG_REF_FIXTURE,
    ] {
        assert_produces_valid_pdf(fixture);
    }
}

#[test]
fn acceptance_pr_188_contributor_acceptance_page_fill_reset() {
    let data = load_fixture(PR_188_PAGE_FILL_FIXTURE);
    let (document, _warnings) = PptxParser
        .parse(&data, &ConvertOptions::default())
        .expect("fixture should parse");
    let output = generate_typst(&document).expect("fixture should generate Typst");
    let page_settings = output
        .source
        .lines()
        .filter(|line| line.starts_with("#set page("))
        .collect::<Vec<_>>();

    assert_eq!(page_settings.len(), 2);
    assert!(page_settings[0].contains("fill: rgb(192, 0, 0)"));
    assert!(page_settings[1].contains("fill: white"));
}

// ---------------------------------------------------------------------------
// paragraph-boundary-spacing.pptx
// ---------------------------------------------------------------------------

#[test]
fn smoke_paragraph_boundary_spacing() {
    assert_produces_valid_pdf("paragraph-boundary-spacing.pptx");
}

#[test]
fn structure_paragraph_boundary_spacing_keeps_three_unstyled_paragraphs() {
    let pages = fixed_pages("paragraph-boundary-spacing.pptx");
    assert_eq!(pages.len(), 1);
    let text_box = pages[0]
        .elements
        .iter()
        .find_map(|element| match &element.kind {
            FixedElementKind::TextBox(text_box) => Some(text_box),
            _ => None,
        })
        .expect("fixture should contain its text frame");
    let paragraphs: Vec<_> = text_box
        .content
        .iter()
        .filter_map(|block| match block {
            Block::Paragraph(paragraph) => Some(paragraph),
            _ => None,
        })
        .collect();

    assert_eq!(paragraphs.len(), 3);
    for (index, paragraph) in paragraphs.iter().enumerate() {
        assert!(
            matches!(
                paragraph.style.line_spacing,
                Some(LineSpacing::Proportional(factor)) if (factor - 1.0).abs() < f64::EPSILON
            ),
            "paragraph should retain its 100% line spacing: {:?}",
            paragraph.style.line_spacing
        );
        assert!(paragraph.runs[0].style.font_family.is_none());
        assert_eq!(paragraph.runs[0].style.font_size, Some(18.0));
        assert!(
            paragraph.runs[0]
                .text
                .starts_with(&format!("Paragraph {}", ["one", "two", "three"][index]))
        );
    }
}

// ---------------------------------------------------------------------------
// minimal.pptx
// ---------------------------------------------------------------------------

#[test]
fn smoke_minimal() {
    assert_produces_valid_pdf("minimal.pptx");
}

#[test]
fn structure_minimal() {
    // minimal.pptx contains only slide layouts/masters but no actual slides
    let data = load_fixture("minimal.pptx");
    let result = PptxParser.parse(&data, &ConvertOptions::default());
    match result {
        Ok((doc, _)) => {
            let slides: Vec<_> = doc
                .pages
                .iter()
                .filter(|p| matches!(p, Page::Fixed(_)))
                .collect();
            // 0 slides is the expected result for this fixture
            assert!(
                slides.is_empty(),
                "minimal.pptx has no actual slides, expected 0 pages"
            );
        }
        Err(_) => {
            // Parse error is also acceptable for a file with no slides
        }
    }
}

// ---------------------------------------------------------------------------
// no-slides.pptx
// ---------------------------------------------------------------------------

#[test]
fn smoke_no_slides() {
    // Must not panic — either empty result or parse error is fine.
    let path = fixture_path("no-slides.pptx");
    let _ = office2pdf::convert(&path);
}

#[test]
fn structure_no_slides() {
    let data = load_fixture("no-slides.pptx");
    match PptxParser.parse(&data, &ConvertOptions::default()) {
        Ok((doc, _)) => {
            // 0 pages is acceptable for a file with no slides
            let slide_count = doc
                .pages
                .iter()
                .filter(|p| matches!(p, Page::Fixed(_)))
                .count();
            assert_eq!(slide_count, 0, "no-slides file should produce 0 pages");
        }
        Err(_) => {
            // Parse error is also acceptable
        }
    }
}

// ---------------------------------------------------------------------------
// powerpoint_sample.pptx
// ---------------------------------------------------------------------------

#[test]
fn smoke_powerpoint_sample() {
    assert_produces_valid_pdf("powerpoint_sample.pptx");
}

#[test]
fn structure_powerpoint_sample() {
    let pages = fixed_pages("powerpoint_sample.pptx");
    assert!(pages.len() >= 2, "should have >= 2 slides");
    assert!(has_textbox_with_content(&pages), "should have text content");
}

// ---------------------------------------------------------------------------
// powerpoint_with_image.pptx
// ---------------------------------------------------------------------------

#[test]
fn smoke_powerpoint_with_image() {
    assert_produces_valid_pdf("powerpoint_with_image.pptx");
}

#[test]
fn structure_powerpoint_with_image() {
    let pages = fixed_pages("powerpoint_with_image.pptx");
    assert!(
        has_fixed_image(&pages),
        "should have FixedElementKind::Image"
    );
}

// ---------------------------------------------------------------------------
// test_slides.pptx
// ---------------------------------------------------------------------------

#[test]
fn smoke_test_slides() {
    assert_produces_valid_pdf("test_slides.pptx");
}

#[test]
fn structure_test_slides() {
    let pages = fixed_pages("test_slides.pptx");
    assert!(!pages.is_empty(), "should have at least 1 slide");
}

// ---------------------------------------------------------------------------
// test.pptx
// ---------------------------------------------------------------------------

#[test]
fn smoke_test() {
    assert_produces_valid_pdf("test.pptx");
}

#[test]
fn structure_test() {
    let pages = fixed_pages("test.pptx");
    assert!(!pages.is_empty(), "should have at least one slide");
    assert!(has_textbox_with_content(&pages), "should have text content");
}

// ===========================================================================
// PDF text content verification
// ===========================================================================

/// Helper: convert a PPTX fixture to PDF and extract text.
fn pdf_text(name: &str) -> String {
    let path = fixture_path(name);
    let result = office2pdf::convert(&path).expect("conversion should succeed");
    common::extract_pdf_text(&result.pdf)
}

// ---------------------------------------------------------------------------
// powerpoint_sample.pptx — text content
// ---------------------------------------------------------------------------

#[test]
fn text_content_powerpoint_sample() {
    let text = pdf_text("powerpoint_sample.pptx");
    assert!(
        text.contains("slide title") || text.contains("Slide Title") || text.contains("Test"),
        "PDF should contain slide title text"
    );
}

// ---------------------------------------------------------------------------
// test.pptx — text content
// ---------------------------------------------------------------------------

#[test]
fn text_content_test() {
    let text = pdf_text("test.pptx");
    assert!(
        text.contains("Presentation Title") || text.contains("Title"),
        "PDF should contain presentation title"
    );
}

// ---------------------------------------------------------------------------
// test_slides.pptx — text content
// ---------------------------------------------------------------------------

#[test]
fn text_content_test_slides() {
    let text = pdf_text("test_slides.pptx");
    assert!(
        text.contains("Test text") || text.contains("Box"),
        "PDF should contain slide text content"
    );
}

// ===========================================================================
// Third-party fixtures — smoke tests (must not panic)
// ===========================================================================

/// Generate a pair of smoke + basic-structure tests for a PPTX fixture.
macro_rules! pptx_fixture_tests {
    ($test_name:ident, $file:expr) => {
        paste::paste! {
            #[test]
            fn [<smoke_ $test_name>]() {
                assert_produces_valid_pdf($file);
            }

            #[test]
            fn [<structure_ $test_name>]() {
                let data = load_fixture($file);
                match PptxParser.parse(&data, &ConvertOptions::default()) {
                    Ok((doc, _)) => {
                        // Just verify parsing succeeds — slide count varies by file
                        let _ = doc.pages.len();
                    }
                    Err(e) => {
                        eprintln!("[WARN] {}: parse error (non-panic): {e}", $file);
                    }
                }
            }
        }
    };
}

// --- CC0 (Public Domain) ---------------------------------------------------

pptx_fixture_tests!(ffc, "ffc.pptx");
pptx_fixture_tests!(one_slide, "1-slide.pptx");
pptx_fixture_tests!(five_slides, "5-slides.pptx");
pptx_fixture_tests!(ten_slides, "10-slides.pptx");

// --- Apache POI (Apache 2.0) -----------------------------------------------

pptx_fixture_tests!(bar_chart, "bar-chart.pptx");
pptx_fixture_tests!(pie_chart, "pie-chart.pptx");
pptx_fixture_tests!(line_chart, "line-chart.pptx");
pptx_fixture_tests!(scatter_chart, "scatter-chart.pptx");
pptx_fixture_tests!(radar_chart, "radar-chart.pptx");
pptx_fixture_tests!(chart_picture_bg, "chart-picture-bg.pptx");
pptx_fixture_tests!(table_test_poi, "table_test.pptx");
pptx_fixture_tests!(table_test2, "table_test2.pptx");
pptx_fixture_tests!(table_with_theme, "table-with-theme.pptx");
pptx_fixture_tests!(backgrounds, "backgrounds.pptx");
pptx_fixture_tests!(themes, "themes.pptx");
pptx_fixture_tests!(smart_art, "SmartArt.pptx");
pptx_fixture_tests!(smart_art_simple, "smartart-simple.pptx");
pptx_fixture_tests!(embedded_audio, "EmbeddedAudio.pptx");
pptx_fixture_tests!(embedded_video, "EmbeddedVideo.pptx");
pptx_fixture_tests!(with_japanese, "with_japanese.pptx");
pptx_fixture_tests!(with_master, "WithMaster.pptx");
pptx_fixture_tests!(comment_45545, "45545_Comment.pptx");
pptx_fixture_tests!(keyframes, "keyframes.pptx");
pptx_fixture_tests!(layouts, "layouts.pptx");
pptx_fixture_tests!(shapes, "shapes.pptx");
pptx_fixture_tests!(custom_geo, "customGeo.pptx");
pptx_fixture_tests!(highlight, "highlight-test-case.pptx");
pptx_fixture_tests!(picture_transparency, "picture-transparency.pptx");
pptx_fixture_tests!(poi_sample, "poi_sample.pptx");
pptx_fixture_tests!(present1, "present1.pptx");
pptx_fixture_tests!(rain, "rain.pptx");
pptx_fixture_tests!(copy_slide_demo, "copy-slide-demo.pptx");

// --- MIT: Open-Xml-PowerTools (Microsoft) ----------------------------------

pptx_fixture_tests!(oxp_presentation, "oxp_Presentation.pptx");
pptx_fixture_tests!(oxp_chart_cached, "oxp_CU018-Chart-Cached-Data-41.pptx");
pptx_fixture_tests!(oxp_chart_embedded, "oxp_CU019-Chart-Embedded-Xlsx-41.pptx");
pptx_fixture_tests!(oxp_pb001_input1, "oxp_PB001-Input1.pptx");
pptx_fixture_tests!(oxp_pb001_input2, "oxp_PB001-Input2.pptx");
pptx_fixture_tests!(oxp_pb001_input3, "oxp_PB001-Input3.pptx");
pptx_fixture_tests!(oxp_videos, "oxp_PP006-Videos.pptx");

#[test]
fn smart_art_renders_cached_drawing_shapes() {
    // The SmartArt drawing cache holds five shapes; PowerPoint renders them
    // as blue blocks, but office2pdf produced a blank slide (issue #223).
    let pages = fixed_pages("SmartArt.pptx");
    let shape_count: usize = pages
        .iter()
        .flat_map(|p| p.elements.iter())
        .filter(|e| matches!(e.kind, FixedElementKind::Shape(_)))
        .count();
    assert!(
        shape_count >= 5,
        "SmartArt drawing cache must render its shapes, got {shape_count}"
    );
    // The shapes carry a fill (the accent color), not a blank slide.
    let filled: bool = pages
        .iter()
        .flat_map(|p| p.elements.iter())
        .any(|e| matches!(&e.kind, FixedElementKind::Shape(s) if s.fill.is_some()));
    assert!(filled, "SmartArt shapes must carry their fill color");
}

// ---------------------------------------------------------------------------
// hangul_kinsoku_terminal_punct.pptx — Hangul + trailing punctuation at a
// line boundary (issue #438). Authored and exported by Windows PowerPoint:
// each box holds ten 18pt Malgun Gothic syllables (180pt) in a 183pt-usable
// box, plus one trailing mark, so the mark alone overflows the line.
// ---------------------------------------------------------------------------

/// Runs only where Malgun Gothic exists (the Windows CI runner): without
/// a Hangul-capable font the glyphs never reach the PDF text layer at
/// all, which would vacuously pass the ZWSP check and fail the content
/// check.
#[cfg(target_os = "windows")]
#[test]
fn kinsoku_fixture_text_layer_has_no_zero_width_space() {
    // The break opportunity is carried as U+200B inside the IR, and must
    // never surface in the PDF text layer.
    let text = pdf_text("hangul_kinsoku_terminal_punct.pptx");
    assert!(
        !text.contains('\u{200B}'),
        "zero-width space leaked into the PDF text layer"
    );
    assert!(
        text.contains("가나다라마바사아자차"),
        "fixture text lost during conversion"
    );
}

#[test]
fn kinsoku_break_marker_becomes_inline_box() {
    let data = load_fixture("hangul_kinsoku_terminal_punct.pptx");
    let parser = PptxParser;
    let (document, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let output = generate_typst(&document).unwrap();
    // Ten syllables, then the boxed mark: an inline box is a Contingent
    // Break in UAX #14, so the mark may start the next line instead of
    // dragging the syllable down with it (LB13). Verified against native
    // PowerPoint exports in #438.
    assert!(
        output.source.contains("자차]#box[]"),
        "no break frame before the terminal mark"
    );
    // '%' is the counter-case: PowerPoint keeps it glued to the syllable,
    // so it must never be boxed away from it.
    assert!(
        output.source.contains("차%]"),
        "'%' must stay glued to the syllable, with no break frame between"
    );
    assert!(
        !output.source.contains('\u{200B}'),
        "kinsoku marker leaked into the Typst source"
    );
    // The 09_lecture_ko card replica: the break lands before '?', not
    // between the last two syllables.
    assert!(
        output.source.contains("뜻인가]#box[]"),
        "no break frame in the #438 replica"
    );
}

// ---------------------------------------------------------------------------
// shadow_blur_radii.pptx — outer shadows at blurRad 6/12/24pt plus the #390
// reproduction (9pt), authored and exported by Windows PowerPoint. The GT
// profile behind the ring constants was measured from that export.
// ---------------------------------------------------------------------------

#[test]
fn shadow_blur_renders_gaussian_ring_stack() {
    let data = load_fixture("shadow_blur_radii.pptx");
    let parser = PptxParser;
    let (document, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let output = generate_typst(&document).unwrap();
    // Opacity 40000 (alpha 0.4) solves to this exact ring ladder; one
    // stack per shadowed rectangle, largest blur spans +-2 sigma = 14.4pt.
    for alpha in [9, 23, 37, 32, 16, 6] {
        assert_eq!(
            output
                .source
                .matches(&format!("rgb(0, 0, 0, {alpha})"))
                .count(),
            4,
            "each of the four shadows carries one ring at alpha {alpha}"
        );
    }
    // blur 24pt: sigma 7.2pt, outermost ring outsets 14.4pt each side of
    // the 220x130pt shape.
    assert!(
        output.source.contains("width: 248.8pt, height: 158.8pt"),
        "24pt blur must span +2 sigma"
    );
}
