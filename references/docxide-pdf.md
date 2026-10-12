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

The pinned corpus has 393 inputs/391 references; 376 published cases identify
office2pdf by release version only. Re-run current code.

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
Custom-XML text is retained. Historical feature targets were HTML/MHTML imports,
page borders and line numbering; empty shape text layers need pixel inspection.

This is triage, not a visual pass: missing searchable text can still be painted
as an image/path, and extraction order or font availability can change scores.
Every selected fix still needs fresh native GT, all comparison axes, direct image
inspection, and explicit issue dispositions under our visual contract.
HTML expansion follows its CSS mapping, using Rust [html5ever](https://github.com/servo/html5ever) and
[mailparse](https://github.com/staktrace/mailparse); docx-rs and contextual scans read the expanded XML.
Section line numbering maps starts, intervals and restarts to Typst margin labels.
Case70 native exports check fonts/clearance; compiled tests cover section suppression.

Section alignment maps `sections.rs` properties into Typst page flow. Fresh
Word case69 probes calibrate trailing spacing and continuous-section uniformity;
`both` and mixed automatic pagination warn; mixed explicit shared pages use top.
Numbering follows its legal decimalization. Native case72 probes calibrate
restart ranges, suppression, implicit counters and skipped-level origins; MS-OI29500
2.1.282 confirms Word ignores restart rules inside complete level overrides.
Marker clearance/suffixes follow `pdf/list_label.rs`; native font/size/tab/wrap probes calibrate differences.
