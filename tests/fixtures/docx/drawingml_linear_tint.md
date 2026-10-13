# DrawingML tint probe

Authored rectangle-only DOCX; no images, embedded fonts, or private content.
The theme structure comes from the Apache-2.0
[docxide-pdf case60 fixture](https://github.com/sverrejb/docxide-pdf/blob/1bb140b662a69f8786015c1b532d37d6084d0934/tests/fixtures/cases/case60/input.docx),
with accent1/accent2 replaced by the colors below.

Eight anchored WordprocessingML drawing groups share one paragraph. Each has
one 216 × 72pt rectangle without a stroke. The page is 612 × 792pt;
columns start at x=72/324pt and rows at y=72/216/360/504pt.

Flat RGB values measured from Microsoft Word 16.113.4 PDF export at 150 DPI:

| Input | `a:tint/@val` | Native RGB |
| --- | ---: | --- |
| accent1 `4F81BD` | 60000 | `B2C1DB` |
| accent2 `156082` | 40000 | `CCD2D8` |
| `FF0000` | 50000 | `FFBCBC` |
| `0064FF` | 40000 | `CBD3FF` |
| `000000` | 50000 | `BCBCBC` |
| `01040A` | 60000 | `AAAAAA` |
| `4F81BD` | 0 | `FFFFFF` |
| `156082` | 100000 | `156082` |

A separate PowerPoint 16.113.4 export reproduced these values and measured
`4472C4` at 40000 as `CFD5EA`, and `A5A5A5` at 50000 as `D8D8D8`.
The default DOCX parser test reads this package.
