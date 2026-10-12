# DOCX legal numbering

The unmodified synthetic `tests/fixtures/docx/docxide/legal_numbering.docx` is
upstream case71. Fresh Word 16.113.4 uses Roman parent labels and decimal legal
child references. Before ignores the legal flag; after restores `1.1.`, `1.2.`
and `2.1.`. `compare.jpg` shows native left and filing-time before right.

[Provenance](../../../tests/visual_audits/issue-2082/provenance.json) records hashes
and direct vision findings. The current strict raster report has zero material
clusters and empty [dispositions](../../../tests/visual_audits/issue-2082/cluster-dispositions-page-1.json).
This is not a full visual pass: two shifted paragraphs (three text instances)
and default extraction order remain tracked in #2083; raw text matches.
Small native drift is in #1874.
