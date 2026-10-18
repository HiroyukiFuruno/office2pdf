# Excel plain header/footer anchors

Related: [#1728](https://github.com/developer0hye/office2pdf/issues/1728).

Native Excel starts a plain left section one sheet point to the right of its
floored section margin. Rich sections retain the margin anchor. Centered and
right-aligned sections do not receive this inset.

The byte-identical reported `headerFooterTest.xlsx` archive places both plain
left labels at 55pt over a 54pt margin. Its no-patch re-zip control matches
geometry and every decoded pixel at 150 DPI.

Archived native `1552-footer-batch6-scaled` probes confirm scaling for a footer:
plain Aptos 8pt and 24pt sections start at 18.72pt, while the rich section starts
at 17.939999pt. The effective scale is 0.78 (8pt prints at 6.24pt), despite the
source's cached scale attribute being 85. Both complete control pages match
exact decoded pixels. Only the footer string changes between probe variants.
Header scaling follows the shared section coordinate model; this archive does
not independently verify scaled header placement.

Emit a paint-only horizontal move so the section keeps its existing wrap width.
Keep rich sections and center/right anchors unchanged. Compiled behavioral
controls cover header/footer, all section emission paths, and the fitted case.
The independent header-baseline defect remains #1731.
