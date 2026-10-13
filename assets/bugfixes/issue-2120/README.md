# Floating-picture frame sizing

Independently authored four-band PNG/DOCX, Apache-2.0. Native GT: Microsoft Word
16.113.4 on macOS 26.6.2; one Letter page. Images use pdftoppm at 150 DPI,
progressive JPEG quality 86, stripped metadata and restored 150 DPI density.

Source SHA-256: `165581d19ca274ef24cc3d2e319016ea47cd9ce1f800101c63d6fadf17ed1e0f`.
Native PDF SHA-256: `87cee6d7424f4eea256b64446b3ae1dfe63c764f09aee67dcb158ed697ed1674`.
Current converter SHA-256: `bb94ae39e4b601585344271a649dbf28a51387179eda0267be72052ffe9efcc0`.
Baseline: `b06eeacb`, whose floating-picture generator matches main `b94c1ee3`.
Native packages/PDFs and full-resolution audit crops remain outside git.

Before: the 120×120pt square frame clips a 240×120pt source, losing red/yellow
bands. After: all four bands fill 120×120pt. Compiled regressions also cover
wide frames, all wrap modes, SVGs, crop/rotation and one-dimensional sizing.

The complete after-page/diff/crop inspection still finds ignored source cropping
(#2121) and 13.05pt vertical displacement (#2122). Exact current render-cluster IDs
track those issues. No stroke, dash, hairline, text or emphasis is declared.
Layout/text reports pass but do not measure the image-only displacement;
MuPDF image transforms and inspected crops establish it. This evidence proves
frame sizing, not 393-file visual parity.
