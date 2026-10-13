# DrawingML preset definitions

Unmodified `tools/data/presetShapeDefinitions.xml` from
[sverrejb/docxide-pdf](https://github.com/sverrejb/docxide-pdf/blob/1bb140b662a69f8786015c1b532d37d6084d0934/tools/data/presetShapeDefinitions.xml)
(revision `1bb140b662a69f8786015c1b532d37d6084d0934`), identified there as ECMA-376 Annex D.
The upstream Apache-2.0 license is retained in `LICENSE`.

Cached diagrams use these definitions through our existing guide/path evaluator
when a preset has no native implementation. Presets requiring separate path
paint styles retain their fallback and emit a warning.
Expansion removes a fourth zero padding operand in eight built-in `+-` formulas,
matching upstream's generator. The vendored XML stays unchanged.
