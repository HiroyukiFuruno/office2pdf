# DOCX skipped child levels

`compare.jpg` remains the frozen filing-time comparison: fresh Word 16.113.4
left, pre-restart-fix converter right. `before.jpg` uses `d5f866b6`. Both show
the skipped Item G marker/body 18pt left, with a fallback marker face. Current
output uses its actual third-level style and absolute indentation. The 0.5pt
layout audit, searchable content/order/census and strict raster audit pass.
Sub-material baseline drift (0.201pt maximum) remains in #1874.

[Provenance](../../../tests/visual_audits/issue-2087/provenance.json) records
hashes and calibrated four-level sequences. Their marker style, indent and gaps
are fixed; implicit-ancestor label errors remain in #2091.
