# Progressive JPEG DC-table probe

Authored DOCX with three anchored RGB color grids and no text, fonts, or private content. Images use separate progressive DC-first scans selecting chroma Huffman table 1, 2, or 3. All pictures share one paragraph and page-relative positions on a 612×792pt page:

| DC table | Source pixels | x/y (pt) | Width/height (pt) |
|---|---|---|---|
| 1 | 17×19 | 72/72 | 160/178.824 |
| 2 | 83×59 | 72/300 | 160/113.735 |
| 3 | 160×120 | 72/500 | 160/120 |

No crop, flip, rotation, stroke, or wrapping. JPEG generation and reference measurements are documented in the authored regressions submitted to `etemesi254/zune-image` PR #489. Those fixtures are embedded unchanged under `word/media/`.

Native Word 16.113.4 confirms these positions and colors. Baseline office2pdf
adds 72pt to both image coordinates; that independent defect is tracked in
#2115. Color corruption is tracked in #2114.
