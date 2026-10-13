# Cached DOCX SmartArt recovery

Synthetic case60 from docxide-pdf revision
`1bb140b662a69f8786015c1b532d37d6084d0934`; fixture attribution is in
`tests/fixtures/docx/docxide/README.md`.

- Source SHA-256: `a2dcd3e69ce8c85f2fd2a2dd285e80a6e50c5b85dfe2928e1fc1e2e4c32ec152`.
- GT: Word 16.113.4, macOS 26.6.2, exported 2026-10-13 16:21:55 KST.
- GT PDF SHA-256: `6213569ff7f877a32f13d69db33918ffcd5b1e0e68c70d602171a1277d46df3c`.
- Before: main `ececf77d27f7c3021b43a6d77bc5b4eb08f55d26` production code.
- After: `938eb892f28d2a0b43113c0e266248242b3a4650`.
- After PDF SHA-256: `5f64f5f137a089f4e53cc382349f1d009ca72b206d1c1cc2232a345b891b29db`.
- Both conversions use Word's installed DFonts; page 1, pdftoppm at 150 DPI.

The comparison is fresh; the native export is preserved because Word currently
times out on read-only requests. Baseline output is blank. After restores both
diagrams and all 93 normalized text characters, with no missing/extra trace lines
or painted-text visibility mismatch. This is not a complete visual pass.

Remaining: #2130 tracks the highlight's top offset +5.59849pt and height
-6.950293pt. #1874 tracks fine text shifts (Foo -1.48501pt, upper Lorem
+1.25497pt, upper ipsum +1.19502pt, Amet -0.66960pt), underline placement,
and the native `LoremIpsum` trace group versus two separate output objects.
All current material render clusters are explicitly assigned to those issues.
