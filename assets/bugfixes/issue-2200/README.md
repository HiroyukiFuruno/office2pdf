# Embedded DOCX font key (#2200)

Public synthetic case8 from [docxide-pdf](https://github.com/sverrejb/docxide-pdf/tree/1bb140b662a69f8786015c1b532d37d6084d0934/tests/fixtures/cases/case8),
Apache-2.0. The 7 MB source embeds 16 obfuscated faces, most of them
non-redistributable, so it is not committed. The tracked regression fixture
`tests/fixtures/docx/issue_2200_embedded_press_start_2p.docx` carries its OFL
Press Start 2P part byte for byte.

- Source SHA-256: `b7f679d3cecd44c82db4125b3b1f47d913b1fc5c113794ba54dbe47b17207219`
- GT: the case's `reference.pdf`, `Creator: Microsoft Word`, SHA-256
  `fe4f5428ce5c14d906c1601c27d8586c1167b2b661007d5fba1a2f6371065dd6`; `check_gt_integrity.py --source` reports no corruption.
- Baseline: main `216446b3`, `--profile ci` CLI; before PDF SHA-256
  `35de2e8fe187e1c75d205300031dde6fd11be22a527f9a69fb63ac27644c6a10`.
- After PDF SHA-256: `05b65e4754e85536bd614018864f25d7a7229c174fb5caa734b85463c3678990`.

All images are full Letter page 1 at 150 DPI from `pdftoppm`, encoded as
progressive JPEG quality 86 with metadata stripped and density reset.

Before, `pdffonts` lists C059-Roman and LiberationSans: every pixel-face run
fell back to a serif, and lines moved up to 119.87pt (layout audit at 0.5pt). After, it lists
PressStart2P-Regular and the document's embedded ArialMT, as Word does. All 26 GT
lines match with no missing, extra or re-wrapped text, and the text layers are
identical.

Remaining: the `officia` line draws the face's `ffi` ligature where Word draws
three cells, ending one 10pt cell early (#2205). Pixel-face lines sit
0.52-0.54pt high, and the right-aligned Arial paragraph 0.50pt high with
dx +0.32pt on its first line (#1874).
