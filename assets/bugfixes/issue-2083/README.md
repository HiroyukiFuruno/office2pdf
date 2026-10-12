# DOCX nested-list spacing

`compare.jpg` shows fresh Word 16.113.4 left and filing-time main-equivalent
`b810fc9d` output right, from the synthetic case71 package. The second parent
and final child sit too high. Their inherited after-gap is 200 twips (10pt).
The separate incorrect legal labels visible in this frozen image are #2082.

Issue #2082's current layout report identifies the remaining shifts precisely:
second parent 10.18pt up, final child 20.07pt up. Both and their effect on
default extraction order remain open in #2083, requiring a native spacing probe.
