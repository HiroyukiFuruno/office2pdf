# DOCX nested-list spacing

`compare.jpg` shows fresh Word 16.113.4 left and filing-time main-equivalent
`b810fc9d` output right, from the synthetic case71 package. The second parent
and final child sit too high. Their inherited after-gap is 200 twips (10pt).
The separate incorrect legal labels visible in this frozen image are #2082.

`before.jpg` uses the legal-label fix at `3fe800e3`: the second parent remains
10.18pt up and the final child 20.07pt up. Current `after.jpg` restores both
gaps and default searchable-text order. Native 0/10/20pt after-gap and 20pt
before-gap probes confirm collapsed spacing at each level transition; the
no-patch ZIP control matches the original export.

[Provenance](../../../tests/visual_audits/issue-2083/provenance.json) records
hashes and direct vision findings. The 0.5pt layout audit passes; the strict
raster report has zero material clusters and empty
[dispositions](../../../tests/visual_audits/issue-2083/cluster-dispositions-page-1.json).
Sub-material native baseline drift (0.182pt on this page) remains in #1874.
