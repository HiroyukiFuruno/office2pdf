# Issue #943 visual audit

The fixture is `tests/fixtures/docx/wasm_embedded_cjk.docx`. It contains one
18 pt regular run, `Hello 中文测试文档`, declared as Noto Sans SC and carries an
OFL-licensed subset of that face in `word/fonts/NotoSansSC.odttf`. The source
`NotoSansSC[wght].ttf` was pinned to weight 400 before subsetting, so the
embedded face identifies as `NotoSansSC-Regular` rather than a variable-font
default instance.

`gt.jpg` is the native conversion and `before.jpg` is the unchanged WASM
conversion. Both were rendered with `pdftoppm -r 150` before inspection.

## Before-fix checklist

- Page count and order: one Letter page in both outputs; order matches.
- Element presence: the single text run is present in both outputs, but all six
  CJK glyphs are `.notdef` boxes in WASM.
- Position: the native text bounds are `320x35+153+165`; the WASM bounds are
  `204x28+150+147`. The WASM run is 3 px left and 18 px high at 150 DPI.
- Size: the WASM text block is 116 px narrower and 7 px shorter because it uses
  fallback glyph advances and metrics.
- Rotation and flip: none in either output.
- Fill: black text on a white page in both outputs.
- Stroke, border, and dash style: none.
- Text content: native renders `Hello 中文测试文档`; WASM visibly replaces
  the six CJK characters with boxes. `pdftotext` also drops `中` from the WASM
  output.
- Font family, weight, and style: native embeds `NotoSansSC-Regular`; WASM embeds
  `LibertinusSerif-Regular`. The source run has no bold, italic, or underline.
- Text color: black in both outputs.
- Alignment: both are left aligned; the fallback changes the vertical baseline
  and the total advance.
- Line and paragraph spacing: one line and one paragraph; there is no inter-line
  or inter-paragraph spacing to compare.
- Clipping and overflow: none.

## Fine-detail pass

- A matched `520x170+110+100` crop was inspected at full scale.
- On the lossless 150-DPI PNG renders,
  `magick compare -metric AE -fuzz 5%` reports 6,171 differing pixels. The
  normalized progressive JPEG files stored here report 6,403. Every
  highlighted cluster belongs to the text run and is explained by the wrong
  font plus `.notdef` glyphs; no unrelated cluster exists.
- Hairline inventory: no rules, underlines, borders, tick marks, or other
  elements at or below 1 pt exist in the fixture.
- Weight and emphasis inventory: the only run is regular; there are no bold,
  italic, or underlined runs.

## After-fix checklist

- Page count and order: one Letter page in both outputs; order matches.
- Element presence: the full Latin and CJK run is present in both outputs.
- Position: the native bounds are `320x35+153+165`; WASM is
  `320x35+153+169`. The only remaining difference is a 4 px downward shift,
  equivalent to 2.18016 pt in the PDF text bounds and tracked independently in
  #969.
- Size: width and height match exactly.
- Rotation and flip: none in either output.
- Fill: black text on a white page in both outputs.
- Stroke, border, and dash style: none.
- Text content: both visibly render and extract `Hello 中文测试文档`.
- Font family, weight, and style: both PDFs embed the same subsetted
  `NotoSansSC-Regular` face. The run is regular with no emphasis.
- Text color: black in both outputs.
- Alignment: both are left aligned with identical horizontal bounds and glyph
  advances.
- Line and paragraph spacing: one line and one paragraph; the target-specific
  baseline difference is tracked in #969.
- Clipping and overflow: none.

## After-fix fine-detail pass

- The matched `520x170+110+100` crop was re-inspected at full scale.
- On the lossless 150-DPI PNG renders, whole-page
  `magick compare -metric AE -fuzz 5%` reports 4,476 differing pixels, all
  belonging to the 4 px vertical shift. Moving the WASM PNG up by 4 px makes
  both AE and RMSE exactly zero. The separately encoded progressive JPEGs
  stored here report 4,629 AE before alignment; writing the
  `magick ... -roll +0-4` result back to JPEG leaves 517 AE and RMSE
  `154.617 (0.0023593)` after alignment, confined to JPEG and antialiasing
  differences on the glyph edges. No unrelated cluster remains.
- Hairline inventory: the fixture has no rules, underlines, borders, tick
  marks, or other elements at or below 1 pt.
- Weight and emphasis inventory: the only run is regular; there are no bold,
  italic, or underlined runs.

## Fixture provenance

- Source font SHA-256:
  `a3041811a78c361b1de50f953c805e0244951c21c5bd412f7232ef0d899af0da`
- Pinned and subsetted Regular face SHA-256:
  `11aa3b851dbc6ff32184983acd5eb2d229feb16789eea9d02a602748f2b644a2`
- Deterministic DOCX SHA-256 as audited:
  `0fd41454c4289a535e4fee91dae304e87d73767c59e153f525fedbc32613fd11`.
  #2200 re-obfuscated the face with the correct key; the package is now
  `19b2be17b6f714e9a5d1abeea0ff4c82ec9c1e13b7f1cbc02c03f2c77a46e720`
  and the deobfuscated face is unchanged.
- The license and attribution are recorded in
  `tests/fixtures/THIRD-PARTY-LICENSES.md` and
  `tests/fixtures/licenses/NotoSansSC-OFL.txt`.
