# zune-image

Source: https://github.com/etemesi254/zune-image

Used through `image` and Typst for embedded JPEG decoding.

Upstream commit `2e540da3cbf060d7dc34694657a0b74ddbf10cd6` corrects
non-interleaved progressive DC Huffman-table selection from division to modulo.
Our stable-version backport retains zune-jpeg 0.5.15 and zune-core 0.5.1, including
the previous compiler fix, until a compatible stable release includes both.

Regression coverage: https://github.com/etemesi254/zune-image/pull/489

Three authored grids exercise DC selectors 1–3 and progressive AC refinement.
Sampled RGB expectations come from independent libjpeg-turbo exports, allowing
two channel levels for integer-IDCT rounding. The same images are embedded in
`tests/fixtures/docx/progressive_dc_tables.docx`; no corpus image is redistributed.
