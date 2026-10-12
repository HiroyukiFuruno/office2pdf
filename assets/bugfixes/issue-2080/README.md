# DOCX section body alignment

The synthetic `tests/fixtures/docx/docxide/section_vertical_center.docx` is
upstream case69. Fresh Word 16.113.4 centers its title and two body lines.
Before ignores `w:vAlign`; after preserves the declared alignment and final
paragraph gap. `compare.jpg` shows native left and before right.

[Provenance](../../../tests/visual_audits/issue-2080/provenance.json) records
hashes, native probes and direct vision findings. The four comparison axes pass
at 0.5pt; the strict raster report has zero material clusters and its
[dispositions](../../../tests/visual_audits/issue-2080/cluster-dispositions-page-1.json)
are empty. Sub-material baseline/pitch deltas remain tracked in #1874.
Vertical justification and mixed continuous sections with automatic pagination
remain approximate, with warnings documented in the main README.
