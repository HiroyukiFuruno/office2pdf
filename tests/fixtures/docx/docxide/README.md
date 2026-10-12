# docxide-pdf regression fixtures

`page_borders.docx` is the synthetic `cases/case68/input.docx` from
[docxide-pdf at 1bb140b6](https://github.com/sverrejb/docxide-pdf/blob/1bb140b662a69f8786015c1b532d37d6084d0934/tests/fixtures/cases/case68/input.docx),
licensed under Apache-2.0. It declares four 3pt blue page borders, 24pt from the
page edge. The default parser/render regression reads this tracked package.
`line_numbering.docx` is the synthetic `cases/case70/input.docx` from the same
revision and license. Its twelve Cambria body lines use continuous line numbering;
native Word displays labels 2–13. A default conversion regression reads it.
No fonts or reference PDFs are copied from the upstream corpus.
