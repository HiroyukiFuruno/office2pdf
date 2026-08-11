use super::*;

#[test]
fn test_fixed_page_text_box() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![make_text_box(100.0, 200.0, 300.0, 50.0, "Slide Title")],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("Slide Title"));
    assert!(output.source.contains("100pt"));
    assert!(output.source.contains("200pt"));
}

#[test]
fn test_fixed_page_text_box_uses_padding_and_center_vertical_align() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![make_fixed_text_box(
            100.0,
            200.0,
            300.0,
            50.0,
            Insets {
                top: 3.6,
                right: 7.2,
                bottom: 3.6,
                left: 7.2,
            },
            crate::ir::TextBoxVerticalAlign::Center,
            vec![Block::Paragraph(Paragraph {
                style: ParagraphStyle::default(),
                runs: vec![Run {
                    text: "Centered".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                }],
            })],
        )],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output
            .source
            .contains("inset: (top: 3.6pt, right: 7.2pt, bottom: 3.6pt, left: 7.2pt)")
    );
    assert!(output.source.contains("width: 285.6pt"));
    assert!(output.source.contains(
        "#context {\n    let text_box_slack_0 = calc.max(42.8pt - measure(text_box_content_0).height, 0pt)"
    ));
    assert!(output.source.contains("#v(text_box_slack_0 / 2)"));
    assert!(output.source.contains("let text_box_aligned_0 = ["));
}

#[test]
fn test_fixed_page_text_box_multiple_paragraphs_preserve_breaks() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 200.0,
            width: 300.0,
            height: 100.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![
                    Block::Paragraph(Paragraph {
                        style: ParagraphStyle::default(),
                        runs: vec![Run {
                            text: "First item".to_string(),
                            style: TextStyle::default(),
                            href: None,
                            footnote: None,
                        }],
                    }),
                    Block::Paragraph(Paragraph {
                        style: ParagraphStyle::default(),
                        runs: vec![Run {
                            text: "Second item".to_string(),
                            style: TextStyle::default(),
                            href: None,
                            footnote: None,
                        }],
                    }),
                ],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("First item"));
    assert!(output.source.contains("Second item"));
    assert!(
        output.source.find("First item") < output.source.find("Second item"),
        "paragraph order changed: {}",
        output.source
    );
    assert_eq!(
        output
            .source
            .matches("#block(above: 0pt, below: 0pt)")
            .count(),
        2,
        "each paragraph should retain a separate block: {}",
        output.source
    );
}

#[test]
fn test_fixed_page_text_box_ordered_list_preserves_textbox_styling() {
    use crate::ir::List;

    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 200.0,
            width: 300.0,
            height: 100.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::List(List {
                    kind: ListKind::Ordered,
                    items: vec![
                        ListItem {
                            content: vec![Paragraph {
                                style: ParagraphStyle {
                                    line_spacing: Some(LineSpacing::Proportional(1.5)),
                                    ..ParagraphStyle::default()
                                },
                                runs: vec![Run {
                                    text: " First item".to_string(),
                                    style: TextStyle {
                                        font_size: Some(24.0),
                                        ..TextStyle::default()
                                    },
                                    href: None,
                                    footnote: None,
                                }],
                            }],
                            level: 0,
                            start_at: Some(1),
                        },
                        ListItem {
                            content: vec![Paragraph {
                                style: ParagraphStyle {
                                    line_spacing: Some(LineSpacing::Proportional(1.5)),
                                    ..ParagraphStyle::default()
                                },
                                runs: vec![Run {
                                    text: " Second item".to_string(),
                                    style: TextStyle {
                                        font_size: Some(24.0),
                                        ..TextStyle::default()
                                    },
                                    href: None,
                                    footnote: None,
                                }],
                            }],
                            level: 0,
                            start_at: None,
                        },
                    ],
                    level_styles: BTreeMap::from([(
                        0,
                        ListLevelStyle {
                            kind: ListKind::Ordered,
                            numbering_pattern: Some("1.".to_string()),
                            full_numbering: false,
                            marker_text: None,
                            marker_style: None,
                        },
                    )]),
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(!output.source.contains("#enum("));
    assert!(
        output
            .source
            .contains("#text(size: 24pt)[1\\.]#text(size: 24pt)[ First item]")
    );
    assert!(
        output
            .source
            .contains("#text(size: 24pt)[2\\.]#text(size: 24pt)[ Second item]")
    );
    assert!(!output.source.contains("\\\n2. Second item"));
    assert!(output.source.contains("#v(12pt)"));
    assert!(output.source.contains("#set par(leading: 12pt)"));
}

#[test]
fn test_fixed_page_text_box_compact_list_items_use_full_width_blocks() {
    use crate::ir::List;

    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 200.0,
            width: 320.0,
            height: 140.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::List(List {
                    kind: ListKind::Ordered,
                    items: vec![
                        ListItem {
                            content: vec![Paragraph {
                                style: ParagraphStyle::default(),
                                runs: vec![Run {
                                    text: "Long first item that should wrap inside the fixed text box width".to_string(),
                                    style: TextStyle {
                                        font_size: Some(20.0),
                                        ..TextStyle::default()
                                    },
                                    href: None,
                                    footnote: None,
                                }],
                            }],
                            level: 0,
                            start_at: Some(1),
                        },
                        ListItem {
                            content: vec![Paragraph {
                                style: ParagraphStyle::default(),
                                runs: vec![Run {
                                    text: "Long second item that should also wrap inside the fixed text box width".to_string(),
                                    style: TextStyle {
                                        font_size: Some(20.0),
                                        ..TextStyle::default()
                                    },
                                    href: None,
                                    footnote: None,
                                }],
                            }],
                            level: 0,
                            start_at: None,
                        },
                    ],
                    level_styles: BTreeMap::from([(
                        0,
                        ListLevelStyle {
                            kind: ListKind::Ordered,
                            numbering_pattern: Some("1)".to_string()),
                            full_numbering: false,
                            marker_text: None,
                            marker_style: None,
                        },
                    )]),
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                    no_wrap: false,
                auto_fit: false,
            text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();

    assert_eq!(output.source.matches("#block(width: 320pt)[").count(), 2);
}

#[test]
fn test_fixed_page_text_box_compact_list_preserves_hanging_indent() {
    use crate::ir::List;

    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 200.0,
            width: 320.0,
            height: 140.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::List(List {
                    kind: ListKind::Ordered,
                    items: vec![ListItem {
                        content: vec![Paragraph {
                            style: ParagraphStyle {
                                indent_left: Some(36.0),
                                indent_first_line: Some(-36.0),
                                ..ParagraphStyle::default()
                            },
                            runs: vec![Run {
                                text: "Long first item that should wrap under the body text instead of the number".to_string(),
                                style: TextStyle {
                                    font_size: Some(20.0),
                                    ..TextStyle::default()
                                },
                                href: None,
                                footnote: None,
                            }],
                        }],
                        level: 0,
                        start_at: Some(1),
                    }],
                    level_styles: BTreeMap::from([(
                        0,
                        ListLevelStyle {
                            kind: ListKind::Ordered,
                            numbering_pattern: Some("1)".to_string()),
                            full_numbering: false,
                            marker_text: None,
                            marker_style: None,
                        },
                    )]),
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                    no_wrap: false,
                auto_fit: false,
            text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();

    assert!(
        output
            .source
            .contains("#grid(columns: (36pt, 1fr), gutter: 0pt,"),
        "Expected ordered hanging-indent list to use a marker/body grid, got:\n{}",
        output.source,
    );
    assert!(!output.source.contains("hanging-indent: 36pt"));
    assert!(!output.source.contains("tab_advance_1"));
}

#[test]
fn test_fixed_page_text_box_compact_list_preserves_marker_origin_offset() {
    use crate::ir::List;

    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 200.0,
            width: 320.0,
            height: 140.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::List(List {
                    kind: ListKind::Ordered,
                    items: vec![ListItem {
                        content: vec![Paragraph {
                            style: ParagraphStyle {
                                indent_left: Some(54.0),
                                indent_first_line: Some(-36.0),
                                ..ParagraphStyle::default()
                            },
                            runs: vec![Run {
                                text: "Marker origin should stay inset while wrapped lines align to the text column"
                                    .to_string(),
                                style: TextStyle {
                                    font_size: Some(20.0),
                                    ..TextStyle::default()
                                },
                                href: None,
                                footnote: None,
                            }],
                        }],
                        level: 0,
                        start_at: Some(1),
                    }],
                    level_styles: BTreeMap::from([(
                        0,
                        ListLevelStyle {
                            kind: ListKind::Ordered,
                            numbering_pattern: Some("1)".to_string()),
                            full_numbering: false,
                            marker_text: None,
                            marker_style: None,
                        },
                    )]),
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                    no_wrap: false,
                auto_fit: false,
            text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();

    assert!(
        output
            .source
            .contains("inset: (top: 0pt, right: 0pt, bottom: 0pt, left: 18pt)")
    );
    assert!(
        output
            .source
            .contains("#grid(columns: (36pt, 1fr), gutter: 0pt,")
    );
}

#[test]
fn test_fixed_page_text_box_compact_bulleted_list_uses_custom_marker_style() {
    use crate::ir::List;

    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 200.0,
            width: 320.0,
            height: 140.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::List(List {
                    kind: ListKind::Unordered,
                    items: vec![ListItem {
                        content: vec![Paragraph {
                            style: ParagraphStyle {
                                indent_left: Some(22.5),
                                indent_first_line: Some(-22.5),
                                ..ParagraphStyle::default()
                            },
                            runs: vec![Run {
                                text: "Symbol bullet".to_string(),
                                style: TextStyle {
                                    font_family: Some("Pretendard".to_string()),
                                    font_size: Some(14.0),
                                    ..TextStyle::default()
                                },
                                href: None,
                                footnote: None,
                            }],
                        }],
                        level: 0,
                        start_at: None,
                    }],
                    level_styles: BTreeMap::from([(
                        0,
                        ListLevelStyle {
                            kind: ListKind::Unordered,
                            numbering_pattern: None,
                            full_numbering: false,
                            marker_text: Some("è".to_string()),
                            marker_style: Some(TextStyle {
                                font_family: Some("Wingdings".to_string()),
                                font_size: Some(14.0),
                                ..TextStyle::default()
                            }),
                        },
                    )]),
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();

    assert!(!output.source.contains("Wingdings"));
    assert!(output.source.contains("➔"));
    assert!(output.source.contains("tab_advance_1"));
    assert!(output.source.contains("Symbol bullet"));
}

#[test]
fn test_escape_typst_escapes_leading_dash_list_prefix() {
    assert_eq!(escape_typst("- bullet"), "\\- bullet");
}

#[test]
fn test_fixed_page_text_box_dash_bullets_use_generic_list_path() {
    use crate::ir::List;

    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 200.0,
            width: 320.0,
            height: 140.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::List(List {
                    kind: ListKind::Unordered,
                    items: vec![
                        ListItem {
                            content: vec![Paragraph {
                                style: ParagraphStyle {
                                    indent_left: Some(22.5),
                                    indent_first_line: Some(-22.5),
                                    ..ParagraphStyle::default()
                                },
                                runs: vec![Run {
                                    text: "First dash bullet".to_string(),
                                    style: TextStyle {
                                        font_family: Some("Pretendard".to_string()),
                                        font_size: Some(14.0),
                                        ..TextStyle::default()
                                    },
                                    href: None,
                                    footnote: None,
                                }],
                            }],
                            level: 0,
                            start_at: None,
                        },
                        ListItem {
                            content: vec![Paragraph {
                                style: ParagraphStyle {
                                    indent_left: Some(22.5),
                                    indent_first_line: Some(-22.5),
                                    ..ParagraphStyle::default()
                                },
                                runs: vec![Run {
                                    text: "Second dash bullet".to_string(),
                                    style: TextStyle {
                                        font_family: Some("Pretendard".to_string()),
                                        font_size: Some(14.0),
                                        ..TextStyle::default()
                                    },
                                    href: None,
                                    footnote: None,
                                }],
                            }],
                            level: 0,
                            start_at: None,
                        },
                    ],
                    level_styles: BTreeMap::from([(
                        0,
                        ListLevelStyle {
                            kind: ListKind::Unordered,
                            numbering_pattern: None,
                            full_numbering: false,
                            marker_text: Some("-".to_string()),
                            marker_style: Some(TextStyle {
                                font_family: Some("Pretendard".to_string()),
                                font_size: Some(14.0),
                                ..TextStyle::default()
                            }),
                        },
                    )]),
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();

    assert!(output.source.contains("#list("));
    assert!(output.source.contains("marker: ["));
    assert!(!output.source.contains("tab_advance_1"));
}

#[test]
fn test_fixed_page_text_box_compact_list_preserves_soft_line_breaks() {
    use crate::ir::List;

    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 200.0,
            width: 320.0,
            height: 140.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::List(List {
                    kind: ListKind::Ordered,
                    items: vec![ListItem {
                        content: vec![Paragraph {
                            style: ParagraphStyle::default(),
                            runs: vec![Run {
                                text: "Line 1\u{000B}Line 2".to_string(),
                                style: TextStyle {
                                    font_size: Some(20.0),
                                    ..TextStyle::default()
                                },
                                href: None,
                                footnote: None,
                            }],
                        }],
                        level: 0,
                        start_at: Some(1),
                    }],
                    level_styles: BTreeMap::from([(
                        0,
                        ListLevelStyle {
                            kind: ListKind::Ordered,
                            numbering_pattern: Some("1)".to_string()),
                            full_numbering: false,
                            marker_text: None,
                            marker_style: None,
                        },
                    )]),
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();

    assert!(output.source.contains("#linebreak()"));
    assert!(output.source.contains("#set text(size: 20pt"));
    assert!(output.source.contains("leading: 13pt"));
}

#[test]
fn test_fixed_page_text_box_with_width_height() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![make_text_box(50.0, 60.0, 400.0, 100.0, "Sized box")],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("400pt"));
    assert!(output.source.contains("100pt"));
}

#[test]
fn test_fixed_page_text_box_with_solid_fill() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 200.0,
            width: 300.0,
            height: 50.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle::default(),
                    runs: vec![Run {
                        text: "White BG".to_string(),
                        style: TextStyle::default(),
                        href: None,
                        footnote: None,
                    }],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: Some(Color {
                    r: 255,
                    g: 255,
                    b: 255,
                }),
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("fill: rgb(255, 255, 255)"),
        "Expected white fill in output, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_with_fill_and_stroke() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 50.0,
            y: 80.0,
            width: 200.0,
            height: 40.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle::default(),
                    runs: vec![Run {
                        text: "Bordered".to_string(),
                        style: TextStyle::default(),
                        href: None,
                        footnote: None,
                    }],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: Some(Color {
                    r: 200,
                    g: 220,
                    b: 240,
                }),
                opacity: None,
                stroke: Some(BorderSide {
                    width: 1.0,
                    color: Color { r: 0, g: 0, b: 0 },
                    style: BorderLineStyle::Solid,
                }),
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("fill: rgb(200, 220, 240)"),
        "Expected fill color in output, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains("stroke: 1pt + rgb(0, 0, 0)"),
        "Expected stroke in output, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_with_fill_and_opacity() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 10.0,
            y: 20.0,
            width: 150.0,
            height: 30.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle::default(),
                    runs: vec![Run {
                        text: "Semi-transparent".to_string(),
                        style: TextStyle::default(),
                        href: None,
                        footnote: None,
                    }],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: Some(Color {
                    r: 255,
                    g: 255,
                    b: 255,
                }),
                opacity: Some(0.5),
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("fill: rgb(255, 255, 255, 128)"),
        "Expected fill with alpha in output, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_with_polygon_shape_kind() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 50.0,
            y: 80.0,
            width: 200.0,
            height: 60.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle::default(),
                    runs: vec![Run {
                        text: "Arrow Tab".to_string(),
                        style: TextStyle::default(),
                        href: None,
                        footnote: None,
                    }],
                })],
                padding: Insets {
                    top: 3.6,
                    right: 7.2,
                    bottom: 3.6,
                    left: 7.2,
                },
                vertical_align: crate::ir::TextBoxVerticalAlign::Center,
                fill: Some(Color {
                    r: 0,
                    g: 37,
                    b: 154,
                }),
                opacity: None,
                stroke: None,
                shape_kind: Some(ShapeKind::Polygon {
                    vertices: vec![(0.0, 0.0), (0.85, 0.0), (1.0, 0.5), (0.85, 1.0), (0.0, 1.0)],
                }),
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    // Should contain #polygon for the shape background
    assert!(
        output.source.contains("#polygon("),
        "Expected polygon in output for non-rectangular text box, got:\n{}",
        output.source,
    );
    // The fill should be on the polygon, not the block
    assert!(
        output.source.contains("fill: rgb(0, 37, 154)"),
        "Expected fill color on polygon, got:\n{}",
        output.source,
    );
    // Should NOT have fill on the block itself
    let block_line = output
        .source
        .lines()
        .find(|l| l.contains("#block("))
        .expect("Expected #block line");
    assert!(
        !block_line.contains("fill:"),
        "Block should not have fill when shape_kind is set, got:\n{block_line}",
    );
}

#[test]
fn test_fixed_page_text_box_no_fill_no_stroke() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![make_text_box(10.0, 20.0, 150.0, 30.0, "Plain")],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("fill: white"),
        "Expected fill: white for no-background slide, got:\n{}",
        output.source,
    );
    assert!(
        !output.source.contains("stroke:"),
        "Expected no stroke in output, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_uses_place_for_positioning() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![make_text_box(100.0, 200.0, 300.0, 50.0, "Positioned")],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("place("));
}

#[test]
fn test_fixed_page_text_box_no_wrap_centered_text_uses_inline_box() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 120.0,
            width: 220.0,
            height: 40.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle {
                        alignment: Some(Alignment::Center),
                        ..ParagraphStyle::default()
                    },
                    runs: vec![Run {
                        text: "Centered Title".to_string(),
                        style: TextStyle {
                            font_size: Some(28.0),
                            ..TextStyle::default()
                        },
                        href: None,
                        footnote: None,
                    }],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: true,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("clip: false"),
        "Expected clip: false for no-wrap text box, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains("#set align(center)"),
        "Expected centered alignment in output, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains("#box["),
        "Expected inline no-wrap box in output, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_no_wrap_inserts_word_joiners_for_cjk_titles() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 120.0,
            width: 180.0,
            height: 40.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle {
                        alignment: Some(Alignment::Center),
                        ..ParagraphStyle::default()
                    },
                    runs: vec![Run {
                        text: "제안개요".to_string(),
                        style: TextStyle {
                            font_size: Some(28.0),
                            ..TextStyle::default()
                        },
                        href: None,
                        footnote: None,
                    }],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: true,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("제\u{2060}안\u{2060}개\u{2060}요"),
        "Expected no-wrap word joiners in output, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_no_wrap_keeps_latin_text_extractable() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 120.0,
            width: 180.0,
            height: 40.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle {
                        alignment: Some(Alignment::Center),
                        ..ParagraphStyle::default()
                    },
                    runs: vec![Run {
                        text: "Test text".to_string(),
                        style: TextStyle {
                            font_size: Some(28.0),
                            ..TextStyle::default()
                        },
                        href: None,
                        footnote: None,
                    }],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: true,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("Test text"),
        "Expected plain Latin no-wrap text to remain extractable, got:\n{}",
        output.source,
    );
    assert!(
        !output.source.contains('\u{2060}') && !output.source.contains('\u{00A0}'),
        "Expected no invisible joiners or non-breaking spaces for Latin no-wrap text, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_no_wrap_keeps_mixed_script_titles_unbroken() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 120.0,
            width: 320.0,
            height: 40.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle {
                        alignment: Some(Alignment::Center),
                        ..ParagraphStyle::default()
                    },
                    runs: vec![Run {
                        text: "III. 기술부문".to_string(),
                        style: TextStyle {
                            font_size: Some(28.0),
                            ..TextStyle::default()
                        },
                        href: None,
                        footnote: None,
                    }],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: true,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("I\u{2060}I\u{2060}I\u{2060}.")
            && output
                .source
                .contains("\u{00A0}\u{2060}기\u{2060}술\u{2060}부\u{2060}문"),
        "Expected mixed-script no-wrap title to keep the full heading unbreakable, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_no_wrap_preserves_mixed_script_titles_across_runs() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 120.0,
            width: 320.0,
            height: 40.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle {
                        alignment: Some(Alignment::Center),
                        ..ParagraphStyle::default()
                    },
                    runs: vec![
                        Run {
                            text: "III.".to_string(),
                            style: TextStyle {
                                font_size: Some(28.0),
                                ..TextStyle::default()
                            },
                            href: None,
                            footnote: None,
                        },
                        Run {
                            text: " 기술부문".to_string(),
                            style: TextStyle {
                                font_size: Some(40.0),
                                ..TextStyle::default()
                            },
                            href: None,
                            footnote: None,
                        },
                    ],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: true,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("I\u{2060}I\u{2060}I\u{2060}.")
            && output
                .source
                .contains("\u{00A0}\u{2060}기\u{2060}술\u{2060}부\u{2060}문"),
        "Expected mixed-script no-wrap title to stay unbroken across runs, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_auto_fit_short_text_uses_scale_to_fit() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 120.0,
            width: 145.0,
            height: 12.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle {
                        alignment: Some(Alignment::Center),
                        ..ParagraphStyle::default()
                    },
                    runs: vec![Run {
                        text: "Server(Cloud VM)".to_string(),
                        style: TextStyle {
                            font_size: Some(18.0),
                            ..TextStyle::default()
                        },
                        href: None,
                        footnote: None,
                    }],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: true,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("let text_box_scale_width_0 = (145pt / calc.max(measure(text_box_raw_0).width, 1pt)) * 100%"),
        "Expected width scale calculation, got:\n{}",
        output.source,
    );
    assert!(
        output
            .source
            .contains("let text_box_scale_height_0 = (12pt / 21.599999999999998pt) * 100%"),
        "Expected estimated line-height scale calculation, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains("let text_box_scale_0 = calc.min(100%, calc.min(text_box_scale_width_0, text_box_scale_height_0))"),
        "Expected combined width/height scale clamp, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains(
            "#scale(x: text_box_scale_0, y: text_box_scale_0, origin: top + left, reflow: true)["
        ),
        "Expected scale-to-fit wrapper, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains("#align(center)["),
        "Expected center alignment wrapper, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_no_wrap_auto_fit_uses_scale_to_fit() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 295.0,
            y: 78.0,
            width: 143.16,
            height: 58.15,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle::default(),
                    runs: vec![
                        Run {
                            text: "- ".to_string(),
                            style: TextStyle {
                                font_size: Some(41.99),
                                ..TextStyle::default()
                            },
                            href: None,
                            footnote: None,
                        },
                        Run {
                            text: "목 차 ".to_string(),
                            style: TextStyle {
                                font_size: Some(41.99),
                                ..TextStyle::default()
                            },
                            href: None,
                            footnote: None,
                        },
                        Run {
                            text: "-".to_string(),
                            style: TextStyle {
                                font_size: Some(41.99),
                                ..TextStyle::default()
                            },
                            href: None,
                            footnote: None,
                        },
                    ],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: true,
                auto_fit: true,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("#let text_box_raw_0 = ["),
        "Expected no-wrap auto-fit title to use raw single-line measurement, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains("let text_box_scale_width_0 ="),
        "Expected no-wrap auto-fit title to compute width scale, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains("let text_box_scale_height_0 ="),
        "Expected no-wrap auto-fit title to compute height scale, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains(
            "#scale(x: text_box_scale_0, y: text_box_scale_0, origin: top + left, reflow: true)["
        ),
        "Expected no-wrap auto-fit title to use scale-to-fit, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_mixed_font_header_uses_scale_to_fit() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 0.0,
            y: 2.4,
            width: 474.5,
            height: 57.9,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle::default(),
                    runs: vec![
                        Run {
                            text: "3. 시스템 연동 방안".to_string(),
                            style: TextStyle {
                                font_size: Some(25.0),
                                ..TextStyle::default()
                            },
                            href: None,
                            footnote: None,
                        },
                        Run {
                            text: "| 클라우드 기반 업무 시스템 연동".to_string(),
                            style: TextStyle {
                                font_size: Some(16.0),
                                ..TextStyle::default()
                            },
                            href: None,
                            footnote: None,
                        },
                    ],
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Center,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("#let text_box_raw_0 = ["),
        "Expected raw single-line content wrapper, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains(
            "#scale(x: text_box_scale_0, y: text_box_scale_0, origin: top + left, reflow: true)["
        ),
        "Expected mixed-font header to use scale-to-fit, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_mixed_font_header_with_tight_leading_uses_scale_to_fit() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 0.0,
            y: 2.4,
            width: 474.5,
            height: 57.9,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle {
                        line_spacing: Some(LineSpacing::Proportional(0.585)),
                        ..ParagraphStyle::default()
                    },
                    runs: vec![
                        Run {
                            text: "3. 시스템 연동 방안".to_string(),
                            style: TextStyle {
                                font_size: Some(24.99),
                                ..TextStyle::default()
                            },
                            href: None,
                            footnote: None,
                        },
                        Run {
                            text: "|  클라우드 기반 업무 시스템 연동".to_string(),
                            style: TextStyle {
                                font_size: Some(16.0),
                                ..TextStyle::default()
                            },
                            href: None,
                            footnote: None,
                        },
                    ],
                })],
                padding: Insets {
                    top: 3.6,
                    right: 7.2,
                    bottom: 3.6,
                    left: 7.2,
                },
                vertical_align: crate::ir::TextBoxVerticalAlign::Center,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("#let text_box_raw_0 = ["),
        "Expected mixed-font header to use raw single-line measurement, got:\n{}",
        output.source,
    );
    assert!(
        output
            .source
            .contains("let text_box_scale_0 = calc.min(100%, calc.min(text_box_scale_width_0, text_box_scale_height_0))"),
        "Expected mixed-font header to use combined scale-to-fit, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_wrapped_centered_paragraph_scales_to_fit_height() {
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 368.9,
            y: 376.8,
            width: 139.0,
            height: 58.5,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::Paragraph(Paragraph {
                    style: ParagraphStyle::default(),
                    runs: vec![Run {
                        text: "업무 시스템의 URL 기준으로 문서의 암/복호화 지원".to_string(),
                        style: TextStyle {
                            font_size: Some(18.0),
                            color: Some(Color {
                                r: 255,
                                g: 255,
                                b: 255,
                            }),
                            ..TextStyle::default()
                        },
                        href: None,
                        footnote: None,
                    }],
                })],
                padding: Insets {
                    top: 3.6,
                    right: 7.2,
                    bottom: 3.6,
                    left: 7.2,
                },
                vertical_align: crate::ir::TextBoxVerticalAlign::Center,
                fill: Some(Color {
                    r: 0,
                    g: 120,
                    b: 185,
                }),
                opacity: None,
                stroke: Some(BorderSide {
                    color: Color {
                        r: 0,
                        g: 120,
                        b: 185,
                    },
                    width: 1.0,
                    style: BorderLineStyle::Solid,
                }),
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output
            .source
            .contains("#let text_box_raw_0 = block(width: 124.60000000000001pt)["),
        "Expected wrapped paragraph measurement block, got:\n{}",
        output.source,
    );
    assert!(
        output.source.contains("let text_box_scale_0 = calc.min(100%, (51.3pt / calc.max(measure(text_box_raw_0).height, 1pt)) * 100%)"),
        "Expected height-based wrapped paragraph scale clamp, got:\n{}",
        output.source,
    );
}

#[test]
fn test_fixed_page_text_box_ordered_grid_normalizes_marker_spacing() {
    use crate::ir::List;

    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![FixedElement {
            x: 100.0,
            y: 200.0,
            width: 320.0,
            height: 140.0,
            kind: FixedElementKind::TextBox(crate::ir::TextBoxData {
                content: vec![Block::List(List {
                    kind: ListKind::Ordered,
                    items: vec![
                        ListItem {
                            content: vec![Paragraph {
                                style: ParagraphStyle {
                                    indent_left: Some(36.0),
                                    indent_first_line: Some(-36.0),
                                    ..ParagraphStyle::default()
                                },
                                runs: vec![Run {
                                    text: " Alpha".to_string(),
                                    style: TextStyle {
                                        font_size: Some(20.0),
                                        ..TextStyle::default()
                                    },
                                    href: None,
                                    footnote: None,
                                }],
                            }],
                            level: 0,
                            start_at: Some(1),
                        },
                        ListItem {
                            content: vec![Paragraph {
                                style: ParagraphStyle {
                                    indent_left: Some(36.0),
                                    indent_first_line: Some(-36.0),
                                    ..ParagraphStyle::default()
                                },
                                runs: vec![Run {
                                    text: "Beta".to_string(),
                                    style: TextStyle {
                                        font_size: Some(20.0),
                                        ..TextStyle::default()
                                    },
                                    href: None,
                                    footnote: None,
                                }],
                            }],
                            level: 0,
                            start_at: None,
                        },
                    ],
                    level_styles: BTreeMap::from([(
                        0,
                        ListLevelStyle {
                            kind: ListKind::Ordered,
                            numbering_pattern: Some("1.".to_string()),
                            full_numbering: false,
                            marker_text: None,
                            marker_style: None,
                        },
                    )]),
                })],
                padding: Insets::default(),
                vertical_align: crate::ir::TextBoxVerticalAlign::Top,
                fill: None,
                opacity: None,
                stroke: None,
                shape_kind: None,
                no_wrap: false,
                auto_fit: false,
                text_rotation_deg: None,
            }),
        }],
    )]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("#text(size: 20pt)[1\\. ]"));
    assert!(output.source.contains("#text(size: 20pt)[2\\. ]"));
    assert!(!output.source.contains("#text(size: 20pt)[ Alpha]"));
}

// ----- PowerPoint's line model (issue #513) -----

/// A slide text box holding one paragraph of `text` in `family` at `size`.
fn slide_text_box_source(family: &str, size: f64, style: ParagraphStyle) -> Option<String> {
    crate::render::pdf::powerpoint_line_box_em(family)?;
    let doc = make_doc(vec![make_fixed_page(
        960.0,
        540.0,
        vec![make_fixed_text_box(
            100.0,
            100.0,
            400.0,
            200.0,
            Insets::default(),
            crate::ir::TextBoxVerticalAlign::Top,
            vec![Block::Paragraph(Paragraph {
                style,
                runs: vec![Run {
                    text: text_for(family),
                    style: TextStyle {
                        font_family: Some(family.to_string()),
                        font_size: Some(size),
                        ..TextStyle::default()
                    },
                    href: None,
                    footnote: None,
                }],
            })],
        )],
    )]);
    Some(generate_typst(&doc).unwrap().source)
}

fn text_for(family: &str) -> String {
    format!("Slide body set in {family}")
}

#[test]
fn slide_text_takes_powerpoints_flat_1_2em_line() {
    // PowerPoint gives every line 1.2 times the font size whatever the font's
    // own metrics say. Measured on native exports from both platforms: one
    // wrapped Arial paragraph advances 20.3657pt over 14 gaps at 17pt
    // (1.1980em) and 28.8400pt over 18 at 24pt (1.2017em). Slide text used to
    // take Word's hhea pitch, which is up to 4% short per line (issue #513).
    let Some(source) = slide_text_box_source("Libertinus Serif", 18.0, ParagraphStyle::default())
    else {
        return; // no font book available (e.g. exotic CI sandbox)
    };
    let (top, bottom) =
        emitted_line_box_em(&source).unwrap_or_else(|| panic!("no line box emitted: {source}"));

    assert!(
        (top + bottom - 1.2).abs() < 0.001,
        "a slide line spans 1.2em, got {}em: {source}",
        top + bottom
    );
    assert!(
        source.contains("leading: 0pt"),
        "the advance is carried by the box, not by leading: {source}"
    );
}

#[test]
fn slide_baseline_splits_the_line_at_the_win_ascent_proportion() {
    // The baseline divides that 1.2em in the proportion of OS/2 usWinAscent to
    // usWinAscent + usWinDescent — not the hhea pair, and not Typst's
    // normalised one. Native exports put Arial's first baseline at 0.9718em
    // against 1854/2288 x 1.2 = 0.9724 predicted (issue #513).
    let Some(source) = slide_text_box_source("Libertinus Serif", 18.0, ParagraphStyle::default())
    else {
        return;
    };
    let (top, bottom) = emitted_line_box_em(&source).expect("line box emitted");
    let (expected_top, expected_bottom) =
        crate::render::pdf::powerpoint_line_box_em("Libertinus Serif").expect("metrics resolve");

    assert!(
        (top - expected_top).abs() < 0.001 && (bottom - expected_bottom).abs() < 0.001,
        "expected {expected_top}/{expected_bottom}em, got {top}/{bottom}em: {source}"
    );
    assert!(
        bottom > 0.01,
        "the descent gap must be real: a bottom-anchored box keeps it below its \
         last baseline, and we used to drop it entirely: {source}"
    );
}

#[test]
fn the_split_differs_between_fonts_while_the_line_does_not() {
    // Triangulation: the height is a property of PowerPoint and the split is a
    // property of the font, so two faces must agree on 1.2em and disagree on
    // where the baseline sits inside it.
    let Some(serif) = crate::render::pdf::powerpoint_line_box_em("Libertinus Serif") else {
        return;
    };
    let Some(mono) = crate::render::pdf::powerpoint_line_box_em("DejaVu Sans Mono") else {
        return;
    };

    assert!((serif.0 + serif.1 - 1.2).abs() < 0.000_001);
    assert!((mono.0 + mono.1 - 1.2).abs() < 0.000_001);
    assert!(
        (serif.0 - mono.0).abs() > 0.001,
        "two faces with different usWinAscent ratios should seat the baseline \
         differently: {serif:?} vs {mono:?}"
    );
    assert!(
        serif.0 > 0.0 && serif.1 > 0.0 && mono.0 > 0.0 && mono.1 > 0.0,
        "both edges are positive distances: {serif:?} {mono:?}"
    );
}

#[test]
fn a_slide_paragraph_with_its_own_line_spacing_keeps_it() {
    // `a:lnSpc` is explicit and outranks the default, so the 1.2em box must
    // not be emitted over it.
    let Some(source) = slide_text_box_source(
        "Libertinus Serif",
        18.0,
        ParagraphStyle {
            line_spacing: Some(LineSpacing::Proportional(1.5)),
            ..ParagraphStyle::default()
        },
    ) else {
        return;
    };

    assert!(
        emitted_line_box_em(&source).is_none(),
        "an explicit line spacing must not be overridden by the default line: {source}"
    );
}

#[test]
fn a_slide_paragraph_declares_its_own_block_spacing() {
    // A slide paragraph's gaps are its own `a:spcBef`/`a:spcAft` and nothing
    // else. Leaving them unset let Typst's 1.2em `block.spacing` default in,
    // which put 13pt between the lines of a code block declaring no spacing
    // (issue #513).
    let Some(source) = slide_text_box_source("Libertinus Serif", 11.0, ParagraphStyle::default())
    else {
        return;
    };

    assert!(
        source.contains("above: 0pt, below: 0pt"),
        "an unspaced slide paragraph pins both gaps to zero: {source}"
    );
}
