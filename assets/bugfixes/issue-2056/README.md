# Issue 2056 visual evidence

This directory holds the evidence for the second #2056 fix, which reserves a
header paragraph's exact line height. The wrapped-line evidence from #2057
remains at merge commit `d65372fb7f0172edc1c8940e947c5cc068743bf6`.

Fixture: `tests/fixtures/docx/docxide/first_header_exact_line.docx` is
docxide-pdf's synthetic `cases/first-header-no-minimum-control/input.docx`
(Apache-2.0, revision `fe7b8beafa7612cc89ad62a7d27b2c6141c4d6ed`), unmodified.
Its first-page header has one paragraph with a 160pt exact line 6pt below the
page top. The Word reference is that case's `reference.pdf`: `Creator:
Microsoft Word` (Windows), one 500 x 500pt page. `check_gt_integrity.py
--source` passes. The output is Typst 0.15.1, PDF 1.7.

SHA-256:

- Source: `7bd704d343e6a18e28af36c0289354c9287ae8657fd96e1e33f27e5d4ae722da`
- Word reference: `516625f29371787dd86519c797c639e828af11cd58ca11ba31660d8f40d500ba`
- Before output (main `03ee814e`): `cc696935cf0f518e3eed80fb93f4c7e884c2489b97ced54333cbb6fcf1d1f21f`
- After output: `f54e2c2f809fc36a112e648b80dc9a3126273bda6946f5144177ad0b1e5f6570`

Both CLIs were built with `cargo build --locked --profile ci -p
office2pdf-cli` in separate target directories. The layout report comes from
`compare_layout.py --json --audit --fine-shift 0.5`. The page-1 cluster report
comes from `compare_render.py --dpi 150 --fine-shift 0.5 --strict-clusters`.
The JPEGs are pdftoppm 150 DPI renders, saved as progressive quality 86
without metadata.

Result: Word starts the body at 166pt, the bottom of the exact line. Before the
fix, `Before` sat on the 36pt margin (baseline 47.26pt against Word's
177.35pt). After the fix it sits at 177.26pt. The text layer is identical, with
four lines matched and none missing or extra.

Remaining on this page:

- `Header` baseline 16.27pt against 134.08pt: the drawn story ignores the exact
  line (#2214).
- The header face is Liberation Serif 11pt against Aptos 12pt, and the
  table-cell lines advance 13.8pt against Word's 16pt (`Minimum` -2.30pt,
  `Second` -4.50pt). Both come from the package having no styles part (#2215).
  The pitch is the same on main.

Other files audited on every page:

- `cases/first-header-minimum-row` (Word: 2 pages): main gives one page with
  the 350pt-minimum row on page 1. After the fix there are two pages, page 1's
  `Before` is within 0.1pt, and the row moves to page 2 as in Word. On page 2
  the first-page header is drawn again (#2163), and the row sits 129.99pt low
  because that header, or a first-only story, still sets every page's margin
  (#2216).
- `cases/first-header-small-control` (12pt exact line inside its band): output
  is byte-identical before and after.
- `cases/first-header-no-minimum-overflow` and `first-header-minimum-overflow`
  (not reported in #2056): page 1 now matches Word's body start. Their single
  oversized row already lost 30 of 60 lines on main (#2217). Under the repeated
  header (#2163, #2216), page 2 now keeps 21 of them.
