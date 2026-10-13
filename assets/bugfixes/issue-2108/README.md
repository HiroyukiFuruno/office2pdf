# Damaged PNG conversion evidence

The authored fixture `tests/fixtures/docx/docxide/truncated_png_report.docx`
changes only the PNG tail from its valid control. Fresh Microsoft Word 16.113.4
exported this exact package. Source SHA-256:
`b0a81338fbbbd173435d28e8b8be0074f0a82f06c451e28076884c52d57ed3b2`.
Native PDF SHA-256: `064aef07379269a028a4f31bc472128929c3f2810cdec2e91a06e75d937ca55f`.
The native PDF and local comparison crops remain outside git.

`conversion-failure.json` records a fresh run of main
`b605a5813b0adafadb49587f554a0b2a8d301859`: exit 1, no generated PDF. Its
binary and sanitized diagnostic hashes identify the actual baseline. There is
no before image. The GT/After JPEGs and reports use this fixture, page 1,
`pdftoppm` at 150 DPI, and a 0.5pt fine-detail threshold.

Direct page/diff/crop inspection found the recovered colored rows differ from
Word's black damaged-image preview (#2110); the following paragraph sits
8.01187pt too high (#2109), also present in the valid baseline control. The
remaining small chart-label cluster is glyph-edge rasterization. No missing
text, changed wrapping, page-count, clipping, rotation, border or rectangle
geometry deviation was observed in this authored report.
