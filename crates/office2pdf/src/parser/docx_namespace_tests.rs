use super::*;

const ALTERNATE_PREFIX_DOCUMENT: &[u8] =
    include_bytes!("../../../../tests/fixtures/docx/docxide/alternate_namespace_runs.docx");

fn assert_report_content(document: &Document) {
    let paragraphs: Vec<&Paragraph> = all_blocks(document)
        .iter()
        .filter_map(|block| match block {
            Block::Paragraph(paragraph) => Some(paragraph),
            _ => None,
        })
        .collect();
    assert_eq!(paragraphs.len(), 2);
    for (paragraph, expected) in paragraphs
        .iter()
        .zip(["Quarterly report", "Approved for publication."])
    {
        assert_eq!(
            paragraph
                .runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            expected
        );
        for run in &paragraph.runs {
            assert_eq!(run.style.font_family.as_deref(), Some("Arial"));
            assert_eq!(run.style.font_size, Some(11.0));
        }
    }
    assert_eq!(paragraphs[0].runs[0].style.bold, Some(true));
    assert_ne!(paragraphs[1].runs[0].style.bold, Some(true));
}

#[test]
fn docx_run_namespace_retains_visible_text_and_formatting() {
    let (document, _) = DocxParser
        .parse(ALTERNATE_PREFIX_DOCUMENT, &ConvertOptions::default())
        .unwrap();
    assert_report_content(&document);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn docx_run_namespace_prefixes_keep_the_same_compiled_layout() {
    let conventional: Vec<u8> =
        rewrite_docx_part(ALTERNATE_PREFIX_DOCUMENT, "word/document.xml", |xml| {
            xml.replace("xmlns:ns0=", "xmlns:w=").replace("ns0:", "w:")
        });
    let placements = |data: &[u8]| {
        let (document, _) = DocxParser.parse(data, &ConvertOptions::default()).unwrap();
        assert_report_content(&document);
        let source = crate::render::typst_gen::generate_typst(&document).unwrap();
        crate::render::pdf::compiled_text_runs(&source.source, 0)
            .unwrap()
            .into_iter()
            .map(|run| {
                (
                    run.text,
                    run.family,
                    run.font_size_pt,
                    run.left_pt,
                    run.baseline_pt,
                    run.glyph_x_offsets_pt,
                )
            })
            .collect::<Vec<_>>()
    };
    let expected = placements(&conventional);
    assert!(!expected.is_empty());
    for variant in ["alternate", "default", "strict", "scoped-foreign"] {
        let data = rewrite_docx_part(ALTERNATE_PREFIX_DOCUMENT, "word/document.xml", |xml| {
            match variant {
                "alternate" => xml.to_string(),
                "default" => xml
                    .replace(
                        "<ns0:document",
                        "<ns0:document xmlns=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"",
                    )
                    .replace("<ns0:", "<")
                    .replace("</ns0:", "</"),
                "strict" => xml
                    .replace(
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "http://purl.oclc.org/ooxml/wordprocessingml/main",
                    )
                    .replace("xmlns:ns0=", "xmlns:text=")
                    .replace("ns0:", "text:"),
                "scoped-foreign" => xml.replacen(
                    "</ns0:t>",
                    "</ns0:t><f:r xmlns:f=\"urn:metadata\"><f:t>Metadata</f:t></f:r><ns0:t xmlns:ns0=\"urn:metadata\">Shadowed</ns0:t>",
                    1,
                ),
                _ => unreachable!(),
            }
        });
        assert_eq!(placements(&data), expected, "{variant}");
        let text: String = converted_numbering_text(&data);
        assert_eq!(
            text.split_whitespace().collect::<String>(),
            "QuarterlyreportApprovedforpublication.",
            "{variant}: {text}"
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn docx_run_namespace_headers_and_footers_keep_text_and_emphasis() {
    let header = docx_rs::Header::new().add_paragraph(
        docx_rs::Paragraph::new().add_run(
            docx_rs::Run::new()
                .add_text("Approved report header")
                .bold(),
        ),
    );
    let footer = docx_rs::Footer::new().add_paragraph(
        docx_rs::Paragraph::new().add_run(
            docx_rs::Run::new()
                .add_text("Report publication footer")
                .italic(),
        ),
    );
    let docx = docx_rs::Docx::new()
        .header(header)
        .footer(footer)
        .add_paragraph(
            docx_rs::Paragraph::new().add_run(docx_rs::Run::new().add_text("Report body")),
        );
    let mut cursor = Cursor::new(Vec::new());
    docx.build().pack(&mut cursor).unwrap();
    let conventional: Vec<u8> = cursor.into_inner();
    let mut alternate: Vec<u8> = conventional.clone();
    for part in ["word/header1.xml", "word/footer1.xml"] {
        alternate = rewrite_docx_part(&alternate, part, |xml| {
            xml.replace("xmlns:w=", "xmlns:story=")
                .replace("w:", "story:")
        });
    }
    let (document, _) = DocxParser
        .parse(&alternate, &ConvertOptions::default())
        .unwrap();
    let Page::Flow(page) = &document.pages[0] else {
        panic!("flow section")
    };
    for (story, expected, is_bold, is_italic) in [
        (
            page.header.as_ref().unwrap(),
            "Approved report header",
            true,
            false,
        ),
        (
            page.footer.as_ref().unwrap(),
            "Report publication footer",
            false,
            true,
        ),
    ] {
        let runs: Vec<&Run> = story
            .paragraphs
            .iter()
            .flat_map(|paragraph| {
                paragraph
                    .elements
                    .iter()
                    .filter_map(|element| match element {
                        HFInline::Run(run) => Some(run),
                        _ => None,
                    })
            })
            .collect();
        assert_eq!(
            runs.iter().map(|run| run.text.as_str()).collect::<String>(),
            expected
        );
        assert_eq!(runs[0].style.bold.unwrap_or(false), is_bold);
        assert_eq!(runs[0].style.italic.unwrap_or(false), is_italic);
    }
    assert_eq!(
        converted_numbering_text(&alternate),
        converted_numbering_text(&conventional)
    );
}
