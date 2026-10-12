# DOCX restarted line-number start

The synthetic `tests/fixtures/docx/docxide/line_numbering_page_restart.docx`
changes only case70's restart scope. Fresh Word 16.113.4 displays 2–13 for
`start=1`, where pre-fix office2pdf displays 1–12. `compare.jpg` shows native
left and before right; all images are progressive JPEG q86 at 150 DPI.

[Provenance](../../../tests/visual_audits/issue-2076/provenance.json) includes
hashes, direct vision findings and numeric native calibration. [Dispositions](../../../tests/visual_audits/issue-2076/cluster-dispositions-page-1.json)
are empty: the current strict report has zero material clusters. Remaining
native rounding (at most 0.204pt vertically / 0.299pt horizontally) is tracked
in #1874. The selected one-page comparison passes all four axes; the other
probes calibrate starts/restarts without claiming their visual parity.
