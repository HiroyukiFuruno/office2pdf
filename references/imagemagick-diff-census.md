# ImageMagick diff census

Sources: [connected-component labeling](https://imagemagick.org/connected-components/)
and [morphology](https://usage.imagemagick.org/morphology/).

- Connected-component labeling measures adjacent equal-valued pixels; area
  thresholds merge small components into neighboring regions.
- Morphological opening erodes then dilates. A square kernel can remove a long
  narrow rule even when its total differing area is large.
- `scripts/compare_render.py` therefore labels the raw 5%-fuzz mask without
  opening. Existing 4-connectivity and 25px component merging remain; the final
  reporting floor is 20pt² after converting pixel area at the selected DPI.
- This floor bounds the census, not visual materiality. Diagonally disconnected
  hairlines, small missing punctuation, and other sub-floor details still need
  full-resolution inspection. An empty census is not a fidelity verdict.
- Actual ImageMagick tests cover missing one/two-pixel horizontal and vertical
  rules, separated dashes, closed outlines, solid regions, and empty/small masks.
  CI's visual-contract job already installs ImageMagick and runs these tests.
