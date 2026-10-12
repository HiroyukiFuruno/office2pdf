# DOCX disabled line numbering

The synthetic `tests/fixtures/docx/docxide/line_numbering_disabled.docx` removes
case70’s line-number attributes. Fresh Word 16.113.4 renders twelve Cambria
body lines without margin labels. Before inserts 1–12; after preserves the
empty margin. `compare.jpg` shows native left and before right.

[Provenance](../../../tests/visual_audits/issue-2078/provenance.json) records
hashes and direct vision findings. All four comparison axes pass at 0.5pt;
the strict raster report has zero material clusters and its
[dispositions](../../../tests/visual_audits/issue-2078/cluster-dispositions-page-1.json)
are empty. Sub-material native rounding remains tracked in #1874.
