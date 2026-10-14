# Table-style line spacing defect

Public synthetic case15 from [docxide-pdf](https://github.com/sverrejb/docxide-pdf/tree/1bb140b662a69f8786015c1b532d37d6084d0934/tests/fixtures/cases/case15), Apache-2.0. No internal document or font files are submitted.

`compare.jpg`: native Word reference on the left, main `8acc920a2088a9e1c38f3385c23f537ba49f7449` output on the right, page 1 of 1, pdftoppm at 150 DPI. Original 1275x1650 panels are retained without upscaling; JPEG quality 86, progressive, metadata stripped and 150 DPI density.

- Source SHA256: `b5b9db400d391d466e54164c0a044cd46ab040fd4459cce66accc6c5632dbad1`
- Reference SHA256: `cf483f616c2e17713274d2202373dc420e3cc8c78dcb485f399a498d66ac09da`; Creator: Microsoft Word 2024; source-aware integrity PASS.
- Output SHA256: `699990feae006dc0392386cc2ffac56f5fc28013b43c9fbe8432132c248897d4`
- Baseline converter SHA256: `41159e2a1cf26f2a90800089c6fb87e64c417db31269ac939aed3e18048f8d2b`

Reproduce with the pinned public input/reference and a CLI built from the baseline using `cargo build --locked --profile ci -p office2pdf-cli`. Run the baseline executable as `/path/to/baseline/office2pdf input.docx -o output.pdf`, `python3 scripts/check_gt_integrity.py reference.pdf --source input.docx --json`, `python3 scripts/compare_layout.py reference.pdf output.pdf --json --audit --fine-shift 0.5`, and `python3 scripts/compare_text_layer.py reference.pdf output.pdf --json`.

Model vision inspected the full native/main/current pages, their diffs and all five full-width table triplets at original 150 DPI. Automatic rows are taller than Word and progressively move later tables down (worst baseline 23.2934pt, pitch 2.4219pt); both fixed 36pt rows keep their heights. Black body text, blue bold titles, merged-cell divisions and red/black/blue border regions are present and readable. No italic, underline, CJK or dashed borders occur. This is defect evidence, not a passed fidelity audit.

Related: #2147 (TableGrid line240/auto is omitted by the table-style parser); #2148 (independent 5.4pt table-origin error); #1874 (small initial-title placement/advance differences). The character multiset and 51 matched instances agree, but normalized extraction order moves the vertically aligned second-row first-cell label earlier; that observation remains open under #2149. No after-fix image or cluster approval is claimed.
