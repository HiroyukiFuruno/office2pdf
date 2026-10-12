# docxide-pdf

- Repository: https://github.com/sverrejb/docxide-pdf
- Inspected revision: `1bb140b662a69f8786015c1b532d37d6084d0934` (0.18.4)
- License: Apache-2.0; fixture and font redistribution rights require separate review.
- Published comparison: https://sverrejb.github.io/docxide-pdf/

Direct OOXML-to-PDF renderer with a Word-exported reference corpus. Useful
implementation references: `src/docx/alt_chunk.rs` (HTML/MHTML imports),
`sections.rs` (page borders, line numbering, vertical alignment), `numbering.rs`
(restart/override/legal-numbering semantics), and the preset-geometry engine.
Our Typst backend already handles some features this renderer lacks, including
complex-script shaping; copy patterns only after reproducing a missing behavior.

The pinned checkout contains 393 inputs and 391 reference PDFs; its published
comparison contains 376 cases and identifies office2pdf only by release version,
not our commit. Re-run current code instead of treating published failures as live.

```sh
python3 scripts/audit_docxide_corpus.py --corpus-root /path/to/pinned/docxide-pdf \
  --converter /path/to/current/office2pdf --output /path/to/local/audit
```

The runner includes upstream skipped cases and records missing references,
conversion failures/timeouts, source/reference/output hashes, reference metadata,
page counts, and normalized text findings. It never executes fixture generators.
Token identities are hashed; source-document text stays outside the report.
The baseline is `tests/compatibility/docxide-pdf/baseline.json`.

Baseline against main `39bcb289`: 390 comparisons, two missing references,
one image-decoding failure, 64 page-count differences, and 27 empty text layers.
The custom-XML wrapper case retains its text. The baseline identifies HTML/MHTML
imports (`altChunk`), page borders, and line numbering as feature targets; empty text layers
in drawn shapes require pixel inspection before attributing loss.

This is triage, not a visual pass: missing searchable text can still be painted
as an image/path, and extraction order or font availability can change scores.
Every selected fix still needs fresh native GT, all comparison axes, direct image
inspection, and explicit issue dispositions under our visual contract.

HTML import expansion follows its CSS-to-paragraph mapping, with
[Servo html5ever](https://github.com/servo/html5ever) for malformed HTML and
[mailparse](https://github.com/staktrace/mailparse) for MIME decoding. Both are
Rust libraries; expansion feeds the same XML to docx-rs and contextual scans.
