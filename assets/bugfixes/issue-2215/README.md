# Issue 2215 visual evidence

A package with no styles part takes Word's built-in document defaults: Aptos
(the default theme's minor face) at 12pt, with lines spaced at
`w:line="278"` auto.

Source: `tests/fixtures/docx/docxide/first_header_small_control.docx`, the
unmodified docxide-pdf
[`first-header-small-control`](https://github.com/sverrejb/docxide-pdf/tree/fe7b8beafa7612cc89ad62a7d27b2c6141c4d6ed/tests/fixtures/cases/first-header-small-control)
at `fe7b8beafa7612cc89ad62a7d27b2c6141c4d6ed` (synthetic, Apache-2.0). Its
`reference.pdf` reports `Creator: Microsoft Word`, one 500 × 500pt page, and
embeds `Aptos` and `ArialMT`; `check_gt_integrity.py --source` passes.

SHA-256:

- Source: `a4bfd365eefd8a243bd1ceff22f52b95a7722c8d8a51624080a57c0c42add4dc`
- Word reference: `f957cad1324ba1fc370e4f42664e249cb4fa631a1d919e3683c61d9f7d180310`
- Before output (main `68c55ebe`): `cc696935cf0f518e3eed80fb93f4c7e884c2489b97ced54333cbb6fcf1d1f21f`
- After output: `db658d1c49ebd3296a24cd746ec1ab427c119555ca9b776e3fcced1fc785f1a2`

Both CLIs were built with `cargo build --locked --profile ci -p
office2pdf-cli` in separate target directories. `layout-audit.json` comes from
`compare_layout.py --json --audit --fine-shift 0.5`, and
`render-clusters-page-1.json` from `compare_render.py --dpi 150 --fine-shift
0.5 --strict-clusters`. The JPEGs are page 1 rendered by pdftoppm at 150 DPI
and saved as progressive quality 86 without metadata.

Baselines from `mutool draw -F stext`:

| line | Word | before | after |
|---|---:|---:|---:|
| `Header` (header, 12pt exact line) | 15.50pt, Aptos 12 | 16.27pt, Liberation Serif 11 | 18.83pt, Noto Sans 12 |
| `Before` | 47.27pt | 47.26pt | 47.26pt |
| `Minimum` (table cell) | 63.27pt | 61.05pt | 63.24pt |
| `Second` (table cell) | 79.30pt | 74.85pt | 79.22pt |

Remaining, each with its own issue:

- `Header` is drawn 3.33pt below Word's baseline. Its exact 12pt line seats
  the text at the band top (#2214), and the Noto Sans stand-in for Aptos has a
  1.069em seat against Aptos's 0.939em and is 8.5% wider (#2237).

Corpus scan: CI-profile CLIs from main `68c55ebe` and this branch on 187
tracked DOCX fixtures and 393 docxide-pdf cases. 15 tracked and 56 docxide
outputs change, all from packages with no styles part. Inputs that failed on
main fail the same way. Against Word's references, `compare_layout.py` mean
|dy| over the 56 cases falls from 17.16pt to 15.66pt: 34 cases improve and 15
get worse. The worse ones expose independent defects that single spacing used
to offset: #2237 (case83, 86, 87, 125), #2238 (case88–91, 100–102, 118–120)
and #2232 (case95).
