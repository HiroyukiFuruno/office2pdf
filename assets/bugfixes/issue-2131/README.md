# Picture-filled ellipse outlines (#2131)

The tracked reproducer is `tests/fixtures/docx/docxide/smartart_picture_fill.docx`,
a synthetic case59 derivative from docxide-pdf revision
`1bb140b662a69f8786015c1b532d37d6084d0934` (Apache-2.0). All media were replaced
with an authored coordinate grid; no original photographs or font binaries remain.

- Tracked source SHA-256: `8a1b7f56835e8e83b76f36b95882dfa8750fe7ae643f389a7e2744f95ce8a025`.
- Native source SHA-256: `3b8f33ec729c3e419173feef99564cd04b8b5b01c0ff6f48e1d02c66a1397283`.
  Its archive compression differs; all 43 part payloads match the tracked source,
  and the native no-patch control has zero decoded-pixel difference.
- Native export: Word 16.113.4 on macOS 26.6.2, 2026-10-13 18:13:51 KST.
- Native PDF SHA-256: `01be53d385b5669d9c05de4435e81442e1ed6d7cb83c76c97207f961593e16ac`.
- Baseline revision: `4f52dc6b7777f39eb6eb03449016af84489b2cac`.
- Baseline PDF SHA-256: `a4d852b022abfe20a04dbc34daa707edca29f08cda6dbdbb403bc9f1a06f5e61`.
- After PDF SHA-256: `a33c494c0a28f0347a5c265f150bd01683f8bc85c93e7596885b7b3a9a89d940`.

Fresh candidate output is compared with the preserved exact-part-content Word
reference; no refreshed native export is claimed. Both output builds use the
Word application font directory. All images show the full Letter page at 150 DPI,
encoded as progressive JPEG quality 86 with metadata stripped and density reset.
`compare.jpg` preserves the original defect report, GT left/baseline right.

All five authored 2pt white borders now follow their ellipses. Image clipping,
destination stretch, grid alignment, arrow joints, panel rounding and five
searchable serif labels remain intact. Full pages, diff and nine matched crops
were inspected. Layout/text audits pass at 0.5pt; strict render review contains no
material clusters, versus 20 before. Numeric geometry alone is insufficient:
there is one aggregate trace line, with five independently compared text instances.

Remaining sub-material findings are tracked in #1874: label baseline +0.21999pt,
maximum text-width delta 0.238491%, and sparse glyph/panel-edge raster fragments.
These real geometry numbers are not classified as antialiasing. No new material
defect was found on this source; no whole-corpus parity claim is made.
