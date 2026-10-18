# zune-image

Source: https://github.com/etemesi254/zune-image

Used through `image` and Typst for embedded JPEG decoding.

The workspace resolves the crates.io `zune-jpeg` 0.5.15 and `zune-core` 0.5.3
that library users build (#2220). Earlier it was patched to an upstream `dev`
snapshot, not to 0.5.15 plus fixes:

- `zune-core` 0.5.2 made the no-op log macros break 0.5.15's trailing `warn!`
  (#816, upstream #423); 0.5.3 fixed that.
- Upstream commit `2e540da3cbf060d7dc34694657a0b74ddbf10cd6` corrects
  non-interleaved progressive DC Huffman-table selection from division to
  modulo. The division arrived on `dev` in `96b4dbfd` and reached us through
  the snapshot (#2114); 0.5.15 selects with `& (MAX_COMPONENTS - 1)`.
- Baseline non-interleaved decoding (upstream #419, `0346b875`) is fixed only on
  `dev`, so 0.5.15 renders such JPEGs as green stripes until a stable release
  ships it (#2241).

Regression coverage: https://github.com/etemesi254/zune-image/pull/489

Three authored grids exercise DC selectors 1–3 and progressive AC refinement.
Sampled RGB expectations come from independent libjpeg-turbo exports, allowing
two channel levels for integer-IDCT rounding. The same images are embedded in
`tests/fixtures/docx/progressive_dc_tables.docx`; no corpus image is redistributed.
