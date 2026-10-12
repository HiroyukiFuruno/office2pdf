# DOCX section page borders

Native GT: Microsoft Word 16.113.4 on macOS 26.6.2, exported from the same
tracked synthetic fixture as both office2pdf outputs. `compare.jpg` preserves
Word left and pre-fix output right. Images are 150-DPI progressive JPEG q86.

[Source/native/output provenance](../../../tests/visual_audits/issue-2070/provenance.json)
and [current per-ID dispositions](../../../tests/visual_audits/issue-2070/cluster-dispositions-page-1.json)
record the exact inputs and findings. The before build is main `39bcb289`.

The after image restores all four blue edges. Native filled border bands are
2.88pt versus the declared/output 3pt, a 0.12pt (4%) sub-material stroke-weight
difference recorded under #1874; this is geometry, not antialiasing. The only
material diff cluster is an inspected Cambria glyph edge, not border geometry.
No other material deviation was found on the page.
