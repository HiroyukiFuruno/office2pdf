//! Each run's own `w14:ligatures`, recovered from the raw document XML
//! (issue #2205).
//!
//! docx-rs drops the element from `RunProperty`, so `LigatureRules` can only
//! read the levels above the run. Word writes it on a run when a user picks
//! ligatures in the Font dialog, and keeps it on runs pasted from a document
//! whose defaults turn them on.
//!
//! The scan counts the paragraph sequence of [`super::paragraph_cursor`] and,
//! inside each paragraph, the runs docx-rs hands the converter in document
//! order: every `w:r`, except those in `w:del` and `w:moveFrom`, which
//! `flatten_tracked_changes` drops, and math. A positional run cursor still
//! drifts wherever the two sides count runs differently, which is why
//! `PairKerningRules` leaves the run-level `w:kern` unread. Each scanned run
//! therefore keeps its `w:t` text, and the converter takes a value only from
//! the entry whose text matches its own run. The first mismatch stops the rest
//! of that paragraph, which leaves those runs with their styles' answer
//! rather than another run's.

use std::cell::Cell;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::ir::Ligatures;
use crate::parser::xml_util::{OOXML_XML_VERSION, decode_general_ref};

use super::super::styles::parse_ligatures_value;
use super::paragraph_cursor::{
    ParagraphCursor, ScannedParagraph, attribute_value, is_skipped_subtree,
};

/// A `w:r` as the scan saw it.
#[derive(Debug, Default, Clone, PartialEq)]
pub(in super::super) struct ScannedRun {
    /// The run's `w:t` content with whitespace removed, to check the converter
    /// is on the same run.
    text_key: String,
    /// The run's own `w:rPr/w14:ligatures`, or `None` when it states none.
    ligatures: Option<Ligatures>,
}

/// Subtrees whose runs never reach run conversion. The paragraphs inside them
/// still count, so the paragraph sequence stays the shared one.
fn drops_runs(local_name: &[u8]) -> bool {
    matches!(local_name, b"del" | b"moveFrom" | b"oMath" | b"oMathPara")
}

/// The text a run is matched on: its `w:t` content without whitespace, which
/// docx-rs and the raw part do not always preserve alike.
fn text_key(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars().filter(|ch| !ch.is_whitespace())
}

pub(in super::super) struct RunLigatureContext {
    paragraphs: ParagraphCursor<Vec<ScannedRun>>,
}

impl RunLigatureContext {
    pub(in super::super) fn from_xml(xml: Option<&str>) -> Self {
        Self {
            paragraphs: ParagraphCursor::new(
                "w14:ligatures",
                xml.map(Self::scan).unwrap_or_default(),
            ),
        }
    }

    /// The runs of the next paragraph. Must be called exactly once per
    /// converted `w:p`, with that paragraph's `w:pStyle`.
    pub(in super::super) fn next_paragraph(
        &self,
        style_id: Option<&str>,
    ) -> ParagraphRunLigatures<'_> {
        ParagraphRunLigatures {
            runs: self
                .paragraphs
                .next(style_id)
                .map(|(_, runs)| runs.as_slice())
                .unwrap_or_default(),
            cursor: Cell::new(0),
            has_drifted: Cell::new(false),
        }
    }

    fn scan(xml: &str) -> Vec<ScannedParagraph<Vec<ScannedRun>>> {
        let mut reader = Reader::from_str(xml);
        let mut paragraphs: Vec<ScannedParagraph<Vec<ScannedRun>>> = Vec::new();
        // Each open paragraph with the run-dropping depth it opened at: a text
        // box inside a deleted run is still a paragraph of its own whose runs
        // convert.
        let mut open_paragraphs: Vec<(usize, usize)> = Vec::new();
        // Each open `w:r`, as the paragraph and run its content belongs to, or
        // `None` for one conversion never reaches.
        let mut open_runs: Vec<Option<(usize, usize)>> = Vec::new();
        let mut in_body = false;
        let mut in_paragraph_properties = false;
        let mut in_run_properties = false;
        let mut in_text = false;
        let mut skipped_depth: usize = 0;
        let mut dropped_depth: usize = 0;

        // The run whose own content is being read: the innermost open `w:r`,
        // when it belongs to the innermost open paragraph.
        let current_run = |open_paragraphs: &[(usize, usize)],
                           open_runs: &[Option<(usize, usize)>]|
         -> Option<(usize, usize)> {
            let (paragraph, run) = (*open_runs.last()?)?;
            (open_paragraphs.last()?.0 == paragraph).then_some((paragraph, run))
        };

        loop {
            let (element, is_empty) = match reader.read_event() {
                Ok(Event::Start(element)) => (element, false),
                Ok(Event::Empty(element)) => (element, true),
                Ok(Event::End(element)) => {
                    let name = element.local_name();
                    if skipped_depth > 0 {
                        skipped_depth -= usize::from(is_skipped_subtree(name.as_ref()));
                        continue;
                    }
                    dropped_depth -= usize::from(dropped_depth > 0 && drops_runs(name.as_ref()));
                    match name.as_ref() {
                        b"body" => in_body = false,
                        b"p" if in_body => {
                            open_paragraphs.pop();
                            in_paragraph_properties = false;
                        }
                        b"pPr" => in_paragraph_properties = false,
                        b"r" if in_body => {
                            open_runs.pop();
                            in_run_properties = false;
                        }
                        b"rPr" => in_run_properties = false,
                        b"t" => in_text = false,
                        _ => {}
                    }
                    continue;
                }
                Ok(Event::Text(text)) => {
                    if in_text
                        && let Some((paragraph, run)) = current_run(&open_paragraphs, &open_runs)
                        && let Ok(content) = text.xml_content(OOXML_XML_VERSION)
                    {
                        paragraphs[paragraph].value[run]
                            .text_key
                            .extend(text_key(&content));
                    }
                    continue;
                }
                Ok(Event::GeneralRef(reference)) => {
                    if in_text
                        && let Some((paragraph, run)) = current_run(&open_paragraphs, &open_runs)
                        && let Some(content) = decode_general_ref(&reference)
                    {
                        paragraphs[paragraph].value[run]
                            .text_key
                            .extend(text_key(&content));
                    }
                    continue;
                }
                Ok(Event::Eof) | Err(_) => break,
                _ => continue,
            };
            let name = element.local_name();
            if skipped_depth > 0 || is_skipped_subtree(name.as_ref()) {
                skipped_depth += usize::from(!is_empty && is_skipped_subtree(name.as_ref()));
                continue;
            }
            if drops_runs(name.as_ref()) && !is_empty {
                dropped_depth += 1;
                continue;
            }
            match name.as_ref() {
                b"body" if !is_empty => in_body = true,
                b"p" if in_body => {
                    paragraphs.push(ScannedParagraph {
                        style_id: None,
                        value: Vec::new(),
                    });
                    if !is_empty {
                        open_paragraphs.push((paragraphs.len() - 1, dropped_depth));
                    }
                }
                b"pPr" if !is_empty && !open_paragraphs.is_empty() => {
                    in_paragraph_properties = true;
                }
                b"pStyle" if in_paragraph_properties => {
                    if let Some(&(paragraph, _)) = open_paragraphs.last() {
                        paragraphs[paragraph].style_id = attribute_value(&reader, &element, b"val");
                    }
                }
                b"r" if in_body => {
                    let converted_in: Option<usize> = open_paragraphs
                        .last()
                        .filter(|(_, opened_at)| *opened_at == dropped_depth)
                        .map(|(paragraph, _)| *paragraph);
                    let entry: Option<(usize, usize)> = match (converted_in, open_runs.last()) {
                        (None, _) => None,
                        // A `w:r` nested in its own paragraph's run is part of
                        // that run, as docx-rs reads it.
                        (Some(paragraph), Some(Some((open_paragraph, run))))
                            if *open_paragraph == paragraph =>
                        {
                            Some((paragraph, *run))
                        }
                        (Some(paragraph), _) => {
                            paragraphs[paragraph].value.push(ScannedRun::default());
                            Some((paragraph, paragraphs[paragraph].value.len() - 1))
                        }
                    };
                    if !is_empty {
                        open_runs.push(entry);
                    }
                }
                b"rPr"
                    if !is_empty
                        && !in_paragraph_properties
                        && current_run(&open_paragraphs, &open_runs).is_some() =>
                {
                    in_run_properties = true;
                }
                b"ligatures" if in_run_properties => {
                    if let Some((paragraph, run)) = current_run(&open_paragraphs, &open_runs) {
                        paragraphs[paragraph].value[run].ligatures =
                            attribute_value(&reader, &element, b"val")
                                .and_then(|value| parse_ligatures_value(&value));
                    }
                }
                b"t" if !is_empty && current_run(&open_paragraphs, &open_runs).is_some() => {
                    in_text = true;
                }
                _ => {}
            }
        }

        paragraphs
    }
}

/// One paragraph's scanned runs, handed out one per converted docx-rs run.
pub(in super::super) struct ParagraphRunLigatures<'a> {
    runs: &'a [ScannedRun],
    cursor: Cell<usize>,
    has_drifted: Cell<bool>,
}

impl ParagraphRunLigatures<'_> {
    /// The ligatures `run` states itself, or `None` when it states none or
    /// the scan cannot vouch for it. Must be called once per docx-rs run of the
    /// paragraph, in order, before the run is split.
    pub(in super::super) fn next_run(&self, run: &docx_rs::Run) -> Option<Ligatures> {
        let index: usize = self.cursor.get();
        self.cursor.set(index + 1);
        if self.has_drifted.get() {
            return None;
        }
        let scanned: &ScannedRun = self.runs.get(index)?;
        let matches_run: bool = run
            .children
            .iter()
            .filter_map(|child| match child {
                docx_rs::RunChild::Text(text) => Some(text.text.as_str()),
                _ => None,
            })
            .flat_map(text_key)
            .eq(scanned.text_key.chars());
        if !matches_run {
            self.has_drifted.set(true);
            tracing::debug!(
                run_index = index,
                "run scan drifted from the converted runs; leaving the paragraph's \
                 remaining runs to their styles' ligatures"
            );
            return None;
        }
        scanned.ligatures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STANDARD: Ligatures = Ligatures {
        standard: true,
        contextual: false,
        historical: false,
        discretional: false,
    };

    fn run(text: &str) -> docx_rs::Run {
        docx_rs::Run::new().add_text(text)
    }

    fn scanned(xml: &str) -> Vec<Vec<ScannedRun>> {
        RunLigatureContext::scan(xml)
            .into_iter()
            .map(|paragraph| paragraph.value)
            .collect()
    }

    fn scanned_run(text_key: &str, ligatures: Option<Ligatures>) -> ScannedRun {
        ScannedRun {
            text_key: text_key.to_string(),
            ligatures,
        }
    }

    #[test]
    fn reads_each_runs_own_value_in_order() {
        let xml = r#"<w:document xmlns:w="urn:w" xmlns:w14="urn:w14"><w:body>
          <w:p>
            <w:pPr><w:rPr><w14:ligatures w14:val="all"/></w:rPr></w:pPr>
            <w:r><w:rPr><w14:ligatures w14:val="standard"/></w:rPr><w:t>fine fit</w:t></w:r>
            <w:r><w:t xml:space="preserve"> office </w:t></w:r>
            <w:hyperlink><w:r><w:rPr><w14:ligatures w14:val="none"/></w:rPr><w:t>R&amp;D</w:t></w:r></w:hyperlink>
          </w:p>
          <w:p/>
        </w:body></w:document>"#;

        assert_eq!(
            scanned(xml),
            vec![
                vec![
                    scanned_run("finefit", Some(STANDARD)),
                    scanned_run("office", None),
                    scanned_run("R&D", Some(Ligatures::NONE)),
                ],
                vec![],
            ],
            "the paragraph mark's own w:rPr is not a run's"
        );
    }

    /// `flatten_tracked_changes` drops deleted and moved-away runs, and a
    /// text box's `mc:Fallback` copy never converts, so none of them may take
    /// a slot from the runs after them.
    #[test]
    fn skips_the_runs_conversion_never_reaches() {
        let xml = r#"<w:document xmlns:w="urn:w" xmlns:w14="urn:w14" xmlns:mc="urn:mc" xmlns:m="urn:m"><w:body>
          <w:p>
            <w:del><w:r><w:rPr><w14:ligatures w14:val="none"/></w:rPr><w:delText>gone</w:delText></w:r></w:del>
            <w:moveFrom><w:r><w:t>moved</w:t></w:r></w:moveFrom>
            <m:oMath><m:r><m:t>x</m:t></m:r></m:oMath>
            <w:ins><w:r><w:t>kept</w:t></w:r></w:ins>
            <w:r><mc:AlternateContent>
              <mc:Choice><w:drawing><w:txbxContent>
                <w:p><w:r><w:rPr><w14:ligatures w14:val="none"/></w:rPr><w:t>box</w:t></w:r></w:p>
              </w:txbxContent></w:drawing></mc:Choice>
              <mc:Fallback><w:pict><w:txbxContent>
                <w:p><w:r><w:t>box</w:t></w:r></w:p>
              </w:txbxContent></w:pict></mc:Fallback>
            </mc:AlternateContent></w:r>
            <w:r><w:rPr><w14:ligatures w14:val="standard"/></w:rPr><w:t>after</w:t></w:r>
          </w:p>
        </w:body></w:document>"#;

        assert_eq!(
            scanned(xml),
            vec![
                vec![
                    scanned_run("kept", None),
                    scanned_run("", None),
                    scanned_run("after", Some(STANDARD)),
                ],
                vec![scanned_run("box", Some(Ligatures::NONE))],
            ]
        );
    }

    #[test]
    fn hands_each_run_its_own_value() {
        let context = RunLigatureContext::from_xml(Some(
            r#"<w:document xmlns:w="urn:w" xmlns:w14="urn:w14"><w:body>
              <w:p><w:pPr><w:pStyle w:val="Body"/></w:pPr>
                <w:r><w:t>plain</w:t></w:r>
                <w:r><w:rPr><w14:ligatures w14:val="standard"/></w:rPr><w:t>fluffy</w:t></w:r>
              </w:p>
            </w:body></w:document>"#,
        ));
        let paragraph = context.next_paragraph(Some("Body"));

        assert_eq!(paragraph.next_run(&run("plain")), None);
        assert_eq!(paragraph.next_run(&run("fluffy")), Some(STANDARD));
        assert_eq!(paragraph.next_run(&run("extra")), None, "past the scan");
    }

    /// A run the scan and docx-rs count differently must not hand its value to
    /// another run: the rest of the paragraph keeps its styles' answer.
    #[test]
    fn stops_at_the_first_run_whose_text_differs() {
        let context = RunLigatureContext::from_xml(Some(
            r#"<w:document xmlns:w="urn:w" xmlns:w14="urn:w14"><w:body>
              <w:p>
                <w:r><w:rPr><w14:ligatures w14:val="none"/></w:rPr><w:t>first</w:t></w:r>
                <w:r><w:rPr><w14:ligatures w14:val="standard"/></w:rPr><w:t>second</w:t></w:r>
              </w:p>
            </w:body></w:document>"#,
        ));
        let paragraph = context.next_paragraph(None);

        assert_eq!(paragraph.next_run(&run("second")), None, "out of step");
        assert_eq!(
            paragraph.next_run(&run("second")),
            None,
            "a drifted paragraph hands nothing out, even to a run that matches"
        );
    }

    #[test]
    fn a_paragraph_whose_style_disagrees_hands_nothing_out() {
        let context = RunLigatureContext::from_xml(Some(
            r#"<w:document xmlns:w="urn:w" xmlns:w14="urn:w14"><w:body>
              <w:p><w:pPr><w:pStyle w:val="Title"/></w:pPr>
                <w:r><w:rPr><w14:ligatures w14:val="standard"/></w:rPr><w:t>fit</w:t></w:r>
              </w:p>
            </w:body></w:document>"#,
        ));

        assert_eq!(context.next_paragraph(None).next_run(&run("fit")), None);
    }
}
