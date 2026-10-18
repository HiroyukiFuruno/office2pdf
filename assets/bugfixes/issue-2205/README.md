# DOCX standard-ligature defect (#2205)

Public synthetic case8 from [docxide-pdf](https://github.com/sverrejb/docxide-pdf/tree/1bb140b662a69f8786015c1b532d37d6084d0934/tests/fixtures/cases/case8),
Apache-2.0. No internal document or font files are submitted.

`compare.jpg`: the Word reference on the left; main `64f1d00c16294ddeb1296bf70ab93c6d4566c0aa`
output on the right; page 1 of 1; pdftoppm at 150 DPI. Both 1275x1650 panels
are kept without upscaling. The JPEG is quality 86, progressive, metadata
stripped, with 150 DPI density. The defect became visible only after #2200
loaded the document's embedded Press Start 2P face.

- Source SHA-256: `b7f679d3cecd44c82db4125b3b1f47d913b1fc5c113794ba54dbe47b17207219`
- Reference SHA-256: `fe4f5428ce5c14d906c1601c27d8586c1167b2b661007d5fba1a2f6371065dd6`;
  `Creator: Microsoft Word`; source-aware integrity check passes.
- Output SHA-256: `05b65e4754e85536bd614018864f25d7a7229c174fb5caa734b85463c3678990`
- Converter (`--profile ci` CLI) SHA-256: `8d9387c8290e1ce7aa5b2c5e0a7a1deba9d58eeb0568939eeaf02e440766d29f`

In `culpa qui officia deserunt mollit anim id`, Word draws `f`, `f` and `i` as
three pixel cells. Main draws the face's `ffi` ligature in one cell, so the rest
of the line ends one 10pt cell early (width -2.44%, `compare_layout.py --audit
--fine-shift 0.5`). The text layers are identical. The other lines sit 0.50-0.54pt
high (#1874). This is defect evidence, not a fix.
