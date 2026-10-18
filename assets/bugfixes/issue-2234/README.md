# Issue 2234 visual evidence

A run whose cascade states no `w:sz` (run, character style, paragraph style
chain, `w:docDefaults`) is set at 10pt, as Word sets it, instead of Typst's
11pt. `w:szCs` sizes complex-script text only.

Source: `tests/fixtures/docx/docxide/page_header_ordinary_marker.docx`, the
unmodified docxide-pdf
[`page-header-ordinary-marker`](https://github.com/sverrejb/docxide-pdf/tree/fe7b8beafa7612cc89ad62a7d27b2c6141c4d6ed/tests/fixtures/cases/page-header-ordinary-marker)
at `fe7b8beafa7612cc89ad62a7d27b2c6141c4d6ed` (synthetic, Apache-2.0). Its
`reference.pdf` reports `Creator: Microsoft Word`, one A4 page, and embeds
only `Tahoma`; `check_gt_integrity.py --source` passes.

SHA-256:

- Source: `b40d3b1501b5057e9fcc7b54dc4496f3e3f0f8c4351667f413eaf043024bfe65`
- Word reference: `b7c98fc786e4da606a5f963c183583c6e8d70ede9a800ba7bcd40e9f0cc7b4ae`
- Before output (main `66f20f75`): `95af7fb6acaadf57bd53301d503e678c0fee7b6d053c1311439a17990f36d6d9`
- After output: `d3188a4b39d4788651a7b200e3da85935902200ab4382991475e2ed7efb219a0`

Both CLIs were built with `cargo build --locked --profile ci -p
office2pdf-cli` in separate target directories. `layout-audit.json` comes from
`compare_layout.py --json --audit --fine-shift 0.5`, and
`render-clusters-page-1.json` from `compare_render.py --dpi 150 --fine-shift
0.5 --strict-clusters`. The JPEGs are page 1 rendered by pdftoppm at 150 DPI
and saved as progressive quality 86 without metadata.

From `mutool draw -F stext` (baseline, size, width error against Word):

| line | Word | before | after |
|---|---|---|---|
| `HEADER 1` (first-page header) | 38.25pt, Tahoma 10 | 36.29pt, C059 11, +29.1% | 35.57pt, C059 10, +17.4% |
| `Following section` | 94.03pt, Tahoma 10 | 79.94pt, C059 11, +17.1% | 79.22pt, C059 10, +6.5% |

This host has no Tahoma. Converting with the Tahoma subset that Word embedded
in the reference (`--font-path`) shows how much of the remaining difference
comes from the face. The fixed build puts `HEADER 1` on Word's baseline, and
both widths are within 0.1%. Main, at 11pt, set `HEADER 1` 1.11pt low and
10% wide. Only `Following section` stays 12.03pt high, which is exactly one
Tahoma 10 line (1.0005 + 0.2065em).

Remaining, each with its own issue:

- The serif face, the extra width and 2.68pt/2.78pt of each baseline come
  from the C059 stand-in for the missing Tahoma: an ascent of 0.72em against
  Tahoma's 1.0005em (#2235).
- `Following section` is also 12.07pt high because office2pdf drops the line
  Word gives the empty section-mark paragraph (#2232).

The second reported case, `page-header-frame-marker`, was audited the same
way. Both lines are now 10pt and its widths match those above. Its body line
sits 9.19pt below Word's, because the framed empty mark is laid out in the
flow (#2231), less the stand-in's smaller ascent (#2235).

Corpus scan: CI-profile CLIs from main `66f20f75` and this branch on 174
tracked DOCX fixtures and 393 docxide-pdf cases. The 26 inputs that failed on
main fail the same way. 21 outputs change: 17 docxide cases and 4 tracked
fixtures. In each one, text that states no size anywhere moves from 11pt to
10pt.

Against Word's references, over the 17 docxide cases:

- `compare_layout.py` mean width error falls from 14.4% to 6.8%.
- Mean |dy| rises from 10.73pt to 11.14pt.
- With each reference's embedded Tahoma subset, width error falls from 7.0%
  to 0.1%, and mean |dy| falls from 5.06pt to 4.75pt.

The cases that still get worse expose independent defects that the taller
11pt line used to offset:

- #2230: `clear-break-lines-*` and `fullwidth-float-empty-clear`, where a
  clearing break does not move below the floating table.
- #2227: `leading-picture-image-overflow`, where the inline picture is
  lifted out of its line.

One tracked change is not a size fix. `libreoffice/tdf79272_strictDxa.docx`
states `w:sz="22"` in a Strict-conformance styles part that the reader never
resolves, so its text drops from Typst's coincidental 11pt to 10pt (#2243).
The scan also found #2244 (tab leader face) and #2245 (universal-measure
`w:tcW`), which this change does not affect; their compare images are
committed here.
