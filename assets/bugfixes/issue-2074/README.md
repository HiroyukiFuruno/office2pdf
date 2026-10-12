# DOCX section line numbering

Synthetic fixture: `tests/fixtures/docx/docxide/line_numbering.docx`, attributed
in its directory README. Native GT: Word 16.113.4 on macOS 26.6.2.
`compare.jpg` shows native Word left and pre-fix office2pdf right.
Images are progressive JPEG q86 at 150 DPI.

[Provenance](../../../tests/visual_audits/issue-2074/provenance.json) records
hashes and direct vision findings. [Dispositions](../../../tests/visual_audits/issue-2074/cluster-dispositions-page-1.json)
are empty: current output has no material 5% diff clusters. All twelve missing
margin labels are restored in Cambria. Sub-material native rounding differs by
at most 0.204pt vertically and 0.299pt horizontally, tracked in #1874.

This native page verifies continuous numbering. Compiled tests cover page and
section restarts, intervals, clearance and disabling at continuous section boundaries.
