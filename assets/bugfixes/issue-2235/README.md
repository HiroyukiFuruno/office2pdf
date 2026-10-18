# Issue 2235 visual evidence

A missing Tahoma now resolves through Arial's sans chain (Liberation Sans,
then Arimo, then the generic sans faces). Before, the name had no table entry
and no class token, so it fell to the default serif (C059 on this host,
Libertinus Serif in the compiled tests). It also had no line metrics, so on
main each single-spaced 10pt line advanced only 7.22pt.

Source: `tests/fixtures/docx/docxide/page_header_ordinary_marker.docx`, the
unmodified docxide-pdf
[`page-header-ordinary-marker`](https://github.com/sverrejb/docxide-pdf/tree/fe7b8beafa7612cc89ad62a7d27b2c6141c4d6ed/tests/fixtures/cases/page-header-ordinary-marker)
at `fe7b8beafa7612cc89ad62a7d27b2c6141c4d6ed` (synthetic, Apache-2.0). Its
`reference.pdf` reports `Creator: Microsoft Word`, one A4 page, and embeds
only `Tahoma`. `check_gt_integrity.py --source` finds no corruption.

SHA-256:

- Source: `b40d3b1501b5057e9fcc7b54dc4496f3e3f0f8c4351667f413eaf043024bfe65`
- Word reference: `b7c98fc786e4da606a5f963c183583c6e8d70ede9a800ba7bcd40e9f0cc7b4ae`
- Before output (main `75c7aa59`): `d3188a4b39d4788651a7b200e3da85935902200ab4382991475e2ed7efb219a0`
- After output: `03d7975ab72624025313e01517cac17b6e40496c3d0a854c964a57e08dc8ca81`

Both CLIs were built with `cargo build --locked --profile ci -p
office2pdf-cli` in separate target directories. `layout-audit.json` comes from
`compare_layout.py --json --audit --fine-shift 0.5`, and
`render-clusters-page-1.json` from `compare_render.py --dpi 150 --fine-shift
0.5 --strict-clusters`. The JPEGs are page 1 rendered by pdftoppm at 150 DPI
and saved as progressive quality 86 without metadata.

From `mutool draw -F stext` (face, baseline, width error against Word):

| line | Word | before | after |
|---|---|---|---|
| `HEADER 1` | Tahoma 10, 38.25pt | C059 10, 35.57pt, +17.4% | Liberation Sans 10, 37.73pt, +10.1% |
| `Following section` | Tahoma 10, 94.03pt | C059 10, 79.22pt, +6.5% | Liberation Sans 10, 81.38pt, +1.6% |

Why Liberation Sans and not DejaVu Sans, Verdana's stand-in: over the 13
lines Word set in Tahoma 10 in the issue's ten docxide cases, Liberation
Sans runs 2.9% wide and DejaVu Sans 13.2%. LibreOffice's `VCL.xcu`
substitutes Tahoma with Arial-class faces, and docxide-pdf paints it in
Liberation Sans.

Remaining, each with its own issue:

- Liberation Sans is laid out with its own metrics: a 1.1499em line and a
  0.9380em seat, against Tahoma's 1.2070em and 1.0005em. Its capitals are
  also wider. This accounts for `HEADER 1`'s 0.52pt and its 10.1% width
  (#2247).
- `Following section` also sits 12.07pt high, because office2pdf drops the
  line Word gives the empty section-mark paragraph (#2232). The other 0.58pt
  is #2247.

All ten reported cases were audited the same way. Page counts match. In the
four `picture-padding-*` cases, `Following` moved from 14.93–24.55pt off to
2.05–2.42pt off. With Word's own Tahoma subset passed to `--font-path`,
`picture-padding-no-padding` lands on Word's picture and baseline to within
0.14pt, so its remaining offset is #2247 alone. The other residuals are #2227
(`leading-picture-image-*`, and `picture-padding-mixed`'s picture, which Word
wraps below its leading spaces), #2230 (`fullwidth-float-empty-clear`) and
#2231 (`page-header-frame-marker`).

The fix also pins Tahoma's digit advances (1118/2048em regular, 1304/2048em
bold, from the subsets a Microsoft Word export embeds) in the XLSX reference
table. Without that, a Tahoma 10 workbook priced its columns on Liberation
Sans: 6pt per character where Tahoma gives 5pt.
