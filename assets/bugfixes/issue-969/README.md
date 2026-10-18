# Issue #969 visual audit

The fixture is `tests/fixtures/docx/wasm_embedded_cjk.docx`. It contains one
18 pt regular run, `Hello 中文测试文档`, declared as Noto Sans SC and carries the
subsetted `NotoSansSC-Regular` face in `word/fonts/NotoSansSC.odttf`.

`gt.jpg` is the WASM conversion, `before.jpg` is the native conversion from
parent `main`, and `after.jpg` is the fixed native conversion.
All three PDFs used the same fixture bytes and embedded the same subsetted face;
all images were rendered with `pdftoppm -r 150` before JPEG normalization.

The WASM output is the reference for this target-consistency defect because its
metric lookup and its Typst compiler both consume the same in-memory document
face. Before the fix, native compilation consumed the materialized document
face but native code generation rebuilt a separate default font search and
could not measure it. A fresh Microsoft Word 16.111.3 export was attempted on
2026-08-11, but Word returned AppleEvent timeout `-1712` and left an empty PDF,
so no unverified Word output is presented as ground truth.

## Before-fix checklist

- Page count and order: one Letter page in both outputs; order matches.
- Element presence: the single text run is present in both outputs.
- Position: the reference baseline is 96.78961 pt and native is 94.609439 pt,
  so native places the run 2.180171 pt, or 4 px at 150 DPI, too high. The
  raster ink bounds are otherwise the same `320x35` box at `x=153`.
- Size: width and height match exactly.
- Rotation and flip: none in either output.
- Fill: black text on a white page in both outputs.
- Stroke, border, and dash style: none.
- Text content: both visibly render and extract `Hello 中文测试文档`.
- Font family, weight, and style: both PDFs embed the same subsetted
  `NotoSansSC-Regular` face. The source run is regular.
- Text color: black in both outputs.
- Alignment: both are left aligned, with identical horizontal bounds and glyph
  advances.
- Line and paragraph spacing: there is one line and one paragraph, so there is
  no inter-line pitch to compare; the only deviation is its absolute baseline.
- Clipping and overflow: none.

## Before-fix fine-detail pass

- A matched `520x170+110+100` crop was inspected at full scale.
- On the lossless 150-DPI PNG renders, `magick compare -metric AE -fuzz 5%`
  reports 4,476 differing pixels. At 1% fuzz it reports 4,797, and RMSE is
  `2372.66 (0.0362045)`. Every highlighted cluster belongs to the vertically
  displaced text run; no unrelated cluster exists.
- The stored progressive JPEGs report 4,434 AE at 5% fuzz.
- Hairline inventory: no rules, underlines, borders, tick marks, or other
  elements at or below 1 pt exist in the fixture.
- Weight and emphasis inventory: the only run is regular; there are no bold,
  italic, or underlined runs.

## After-fix checklist

- Page count and order: one Letter page in both outputs; order matches.
- Element presence: the single text run is present in both outputs.
- Position: both baselines are 96.78961 pt and both raster bounds match.
- Size: width and height match exactly.
- Rotation and flip: none in either output.
- Fill: black text on a white page in both outputs.
- Stroke, border, and dash style: none.
- Text content: both visibly render and extract `Hello 中文测试文档`.
- Font family, weight, and style: both embed the same subsetted
  `NotoSansSC-Regular` face; the run remains regular.
- Text color: black in both outputs.
- Alignment: both are left aligned, with identical horizontal bounds and glyph
  advances.
- Line and paragraph spacing: the one available baseline matches exactly.
- Clipping and overflow: none.

## After-fix fine-detail pass

- The matched `520x170+110+100` crop was re-inspected at full scale.
- The lossless renders are pixel-identical: AE is zero at both 5% and 1% fuzz,
  and RMSE is zero. Layout comparison reports 0.00 pt vertical and horizontal
  deviation, and the text-layer census and normalized content are identical.
- `gt.jpg` and `after.jpg` have the same SHA-256,
  `cdffceb550af2a9ff7d59ab2a6dc2f587959f79fcc3196ecb68fb4458c5048b4`.
- Hairline inventory: no elements at or below 1 pt exist.
- Weight and emphasis inventory: the only run is regular; there are no bold,
  italic, or underlined runs.
- No visible deviation remains, so there is no additional defect to track.

## Fixture provenance

- Deterministic DOCX SHA-256 as audited:
  `0fd41454c4289a535e4fee91dae304e87d73767c59e153f525fedbc32613fd11`.
  #2200 re-obfuscated the face with the correct key; the package is now
  `19b2be17b6f714e9a5d1abeea0ff4c82ec9c1e13b7f1cbc02c03f2c77a46e720`
  and the deobfuscated face is unchanged.
- Embedded Regular face SHA-256 after deobfuscation:
  `11aa3b851dbc6ff32184983acd5eb2d229feb16789eea9d02a602748f2b644a2`
- The license and attribution are recorded in
  `tests/fixtures/THIRD-PARTY-LICENSES.md` and
  `tests/fixtures/licenses/NotoSansSC-OFL.txt`.
