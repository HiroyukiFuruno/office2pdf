# DOCX HTML import

Original synthetic fixture: `tests/fixtures/docx/altchunk_html.docx`.
Native GT: Word 16.113.4 on macOS 26.6.2. `compare.jpg` shows Word left,
pre-fix office2pdf right. Images use 150-DPI progressive JPEG q86.

[Provenance](../../../tests/visual_audits/issue-2072/provenance.json) records
source/PDF hashes and direct vision findings; [dispositions](../../../tests/visual_audits/issue-2072/cluster-dispositions-page-1.json)
are empty because the current 5% diff has no material clusters. The full sentence
and bold span are restored without any other material deviation on this page.

This page verifies content import. It is not a claim of general HTML/CSS parity
or a visual pass for the external corpus.
