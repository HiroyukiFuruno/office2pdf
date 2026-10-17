use super::*;

/// A chartsheet part with whatever print settings the caller wants, in the
/// element order `CT_Chartsheet` fixes.
fn chartsheet_xml(print_settings: &str) -> String {
    format!(
        r#"<chartsheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheetPr/>
  <sheetViews><sheetView zoomScale="88" workbookViewId="0" zoomToFit="1"/></sheetViews>
  {print_settings}
  <drawing r:id="rId1"/>
</chartsheet>"#
    )
}

#[test]
fn a_chartsheet_stating_nothing_prints_letter_landscape() {
    // `paperSize` unstated means the schema's default of 1, US Letter (issue
    // #717); `orientation` unstated means landscape for a chartsheet.
    let setup = parse_chartsheet_print_setup(&chartsheet_xml(""));
    assert_eq!((setup.size.width, setup.size.height), (792.0, 612.0));
}

#[test]
fn an_explicit_default_orientation_still_prints_landscape() {
    // "default" is the schema's own value for the attribute, so it has to read
    // the same as an absent one.
    let setup =
        parse_chartsheet_print_setup(&chartsheet_xml(r#"<pageSetup orientation="default"/>"#));
    assert_eq!((setup.size.width, setup.size.height), (792.0, 612.0));
}

#[test]
fn a_chartsheet_asking_for_portrait_gets_it() {
    let setup = parse_chartsheet_print_setup(&chartsheet_xml(
        r#"<pageSetup paperSize="9" orientation="portrait"/>"#,
    ));
    assert_eq!((setup.size.width, setup.size.height), (595.0, 842.0));
}

#[test]
fn a_declared_paper_size_turns_on_its_side_with_the_chartsheet() {
    // A5 (code 11) is 420 x 595 upright.
    let setup = parse_chartsheet_print_setup(&chartsheet_xml(r#"<pageSetup paperSize="11"/>"#));
    assert_eq!((setup.size.width, setup.size.height), (595.0, 420.0));
}

#[test]
fn declared_margins_are_read_in_inches() {
    let setup = parse_chartsheet_print_setup(&chartsheet_xml(
        r#"<pageMargins left="1.5" right="0.5" top="1" bottom="0.25" header="0.3" footer="0.3"/>"#,
    ));
    assert_eq!(
        (
            setup.margins.left,
            setup.margins.right,
            setup.margins.top,
            setup.margins.bottom
        ),
        (108.0, 36.0, 72.0, 18.0)
    );
}

#[test]
fn absent_margins_fall_back_to_excels_defaults() {
    let setup = parse_chartsheet_print_setup(&chartsheet_xml(""));
    assert_eq!(
        (
            setup.margins.left,
            setup.margins.right,
            setup.margins.top,
            setup.margins.bottom
        ),
        (50.4, 50.4, 54.0, 54.0)
    );
}

#[test]
fn only_the_chartsheets_of_a_workbook_are_reported() {
    // `chart_sheet.xlsx` declares Sheet1, Sheet2, Chart1 and Sheet3, of which
    // only Chart1 resolves through a `chartsheet` relationship.
    let data = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/xlsx/chart_sheet.xlsx"),
    )
    .expect("fixture should exist");
    let setups = chartsheet_print_setups(&data);
    assert_eq!(setups.keys().collect::<Vec<&String>>(), vec!["Chart1"]);
}

// ----- Where the chart prints inside the page (issue #1147) -----

/// One measured export's print setup, built from its paper and margins rather
/// than from a part, so a case reads as the table row it is.
fn print_setup(page: (f64, f64), margins: (f64, f64, f64, f64)) -> ChartsheetPrintSetup {
    let (width, height) = page;
    let (left, right, top, bottom) = margins;
    ChartsheetPrintSetup {
        size: PageSize { width, height },
        margins: Margins {
            left,
            right,
            top,
            bottom,
        },
    }
}

/// The chart box in page coordinates, which is what `mutool draw -F trace`
/// reads off a native export.
fn printed_box(page: (f64, f64), margins: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let setup: ChartsheetPrintSetup = print_setup(page, margins);
    let placement: ChartsheetChartBox = printed_chart_box(&setup);
    (
        setup.margins.left + placement.x_offset_pt,
        setup.margins.top + placement.y_offset_pt,
        placement.width,
        placement.height,
    )
}

/// Excel seats the chart 4pt inside the margin, on the whole point, rather
/// than starting it at the margin.
///
/// Measured on Excel for Mac 16.100 exports of the `Chart` sheet of
/// `tests/fixtures/xlsx/any_sheets.xlsx`, reading the chart area's `fill_path`
/// out of `mutool draw -F trace`: 58 exports over four papers, both
/// orientations and each margin swept in 0.01in steps put the box origin at
/// `floor(margin) + 4` without exception (issue #1147).
#[test]
fn the_chart_starts_four_points_inside_the_margin() {
    let (x, y, _, _) = printed_box((842.0, 595.0), (50.4, 50.4, 54.0, 54.0));
    assert_eq!((x, y), (54.0, 58.0));
}

/// The margin snaps down to a whole point before the 4pt is added, the way a
/// worksheet's does (issue #1191), so its fraction never reaches the chart.
///
/// Triangulation against the sweeps: left margins of 0.71in (51.12pt) and
/// 0.72in (51.84pt) both export at x 55 and 0.73in (52.56pt) at x 56, which
/// rounding cannot produce — 51.12 rounds to 51 and 51.84 to 52.
#[test]
fn a_fractional_margin_snaps_down_before_the_inset() {
    assert_eq!(
        printed_box((842.0, 595.0), (51.12, 50.4, 54.0, 54.0)).0,
        55.0
    );
    assert_eq!(
        printed_box((842.0, 595.0), (51.84, 50.4, 54.0, 54.0)).0,
        55.0
    );
    assert_eq!(
        printed_box((842.0, 595.0), (52.56, 50.4, 54.0, 54.0)).0,
        56.0
    );
    // The top margin behaves the same: 0.77in (55.44pt) exports at y 59.
    assert_eq!(
        printed_box((842.0, 595.0), (50.4, 50.4, 55.44, 54.0)).1,
        59.0
    );
}

/// The far edges stop short of the printable area's too, so the chart is
/// narrower and shorter than it rather than exactly as wide.
///
/// Excel's own box on this page measures 732.61 x 478.26 inside a 741.2 x 487
/// printable area.
#[test]
fn the_chart_stops_short_of_the_far_margin() {
    let (_, _, width, height) = printed_box((842.0, 595.0), (50.4, 50.4, 54.0, 54.0));
    assert_eq!((width, height), (733.0, 479.0));
}

/// The box follows the paper and every margin, not a constant.
///
/// Each row is one measured export; the expectation is this model's box and
/// the comment beside it is Excel's own, which the model meets to within the
/// internal-grid error term documented on `printed_chart_box`.
#[test]
fn the_box_tracks_the_paper_and_the_margins() {
    // Letter landscape: Excel draws 681.82 x 493.18 at (54, 58).
    assert_eq!(
        printed_box((792.0, 612.0), (50.4, 50.4, 54.0, 54.0)),
        (54.0, 58.0, 683.0, 496.0)
    );
    // A4 portrait: 487.10 x 725.81 at (54, 58).
    assert_eq!(
        printed_box((595.0, 842.0), (50.4, 50.4, 54.0, 54.0)),
        (54.0, 58.0, 486.0, 726.0)
    );
    // A3 landscape: 1083.87 x 725.81 at (54, 58).
    assert_eq!(
        printed_box((1191.0, 842.0), (50.4, 50.4, 54.0, 54.0)),
        (54.0, 58.0, 1082.0, 726.0)
    );
    // A 1.5in left margin: 673.91 x 478.26 at (112, 58).
    assert_eq!(
        printed_box((842.0, 595.0), (108.0, 50.4, 54.0, 54.0)),
        (112.0, 58.0, 675.0, 479.0)
    );
    // The same width of paper spent on the right instead: 673.91 x 478.26 at
    // (54, 58). A right margin moves the far edge, never the origin.
    assert_eq!(
        printed_box((842.0, 595.0), (50.4, 108.0, 54.0, 54.0)),
        (54.0, 58.0, 676.0, 479.0)
    );
    // A 1.5in top margin: 732.69 x 425.00 at (54, 112).
    assert_eq!(
        printed_box((842.0, 595.0), (50.4, 50.4, 108.0, 54.0)),
        (54.0, 112.0, 733.0, 425.0)
    );
    // 1in on every side: 690.00 x 442.00 at (76, 76).
    assert_eq!(
        printed_box((842.0, 595.0), (72.0, 72.0, 72.0, 72.0)),
        (76.0, 76.0, 690.0, 443.0)
    );
    // 2in on every side: 545.21 x 297.26 at (148, 148).
    assert_eq!(
        printed_box((842.0, 595.0), (144.0, 144.0, 144.0, 144.0)),
        (148.0, 148.0, 546.0, 299.0)
    );
}

/// The two exports the far inset misses by the most, kept as the model's
/// stated error rather than left out of the table.
///
/// Half an inch of margin leaves 10.19pt of width unused where Letter portrait
/// leaves 1.54pt in the recorded sweep. This page-only model keeps the
/// difference as its explicit error term rather than fitting one far-edge
/// constant to that sweep (issue #1221).
#[test]
fn the_far_inset_is_the_models_error_term() {
    // A4 landscape, 0.5in margins: Excel draws 755.81 x 511.63 at (40, 40),
    // 6.19pt narrower and 3.37pt shorter than this box.
    assert_eq!(
        printed_box((842.0, 595.0), (36.0, 36.0, 36.0, 36.0)),
        (40.0, 40.0, 762.0, 515.0)
    );
    // Letter portrait: Excel draws 506.06 x 675.76 at (54, 58) — 3.06pt wider
    // than this box, the one direction the model misses the other way.
    assert_eq!(
        printed_box((612.0, 792.0), (50.4, 50.4, 54.0, 54.0)),
        (54.0, 58.0, 503.0, 676.0)
    );
}

/// Margins that leave no paper give an empty box rather than a negative one,
/// which would reach the renderer as a chart drawn backwards.
#[test]
fn margins_wider_than_the_paper_leave_no_chart_box() {
    let (_, _, width, height) = printed_box((842.0, 595.0), (500.0, 500.0, 400.0, 400.0));
    assert_eq!((width, height), (0.0, 0.0));
}
