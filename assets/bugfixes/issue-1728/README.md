# Plain left sheet header/footer anchor

Related: #1728. Remaining findings: #1731, #1874.

The existing tracked Apache POI workbook is Apache-2.0; see
`tests/fixtures/THIRD-PARTY-LICENSES.md`. Images contain only generic labels and
values. No source package, font, personal or organizational content is added.

## Provenance

- Input: `tests/fixtures/xlsx/headerFooterTest.xlsx`, SHA-256
  `db7483ac7bc96d071db57ec6239d2c1b8593a5ef3c7f5b7418df0309753c1fe2`.
- GT: archived native Excel export, 2026-09-15; Creator Excel, macOS 26.6.2
  Quartz PDFContext, one Letter page. Exact exporter version was not recorded.
  PDF SHA-256: `1a5e9319b6cf5b30115a77cb9f9079480afcb6ee80072c7fc7c6465e729b03fd`.
- Source-aware integrity passes. The archived source matches the tracked input
  byte for byte; native no-patch re-zip controls match geometry and exact pixels.
- Before: main `75c7aa5965dbbf4ee74471270c036a7997ce4c37`, PDF SHA-256
  `e0558d568b6d8bc005e4e886b81f1d920689e1e0c289d9709ec1a47fca35ad71`.
- After PDF SHA-256:
  `7c2faafdcfdcb9febbf00c4284e7f9a4126ac941e1f1cd8e1129a66e9e2c4c67`.
- Locked ci-profile CLI hashes, baseline / candidate:
  `40d50b152fa99cbedf19abd1863665d32ca70306659dd9e81a3d649a9a60957d` /
  `6c906e93dc29f9a257b772a5722eea50e14a8c2232d9cf4c6df25a533284cd71`.
- Fresh GT/before/after images are full 150-DPI pages, progressive JPEG quality
  86 with metadata stripped. Existing `compare.jpg` remains historical defect
  evidence; the new trio is the current fix comparison. No new Office export
  is claimed.

## Audit

Both plain left origins now match native at 55pt, previously 54pt. All eight
text instances remain visible and searchable. The normalized/codepoint census
passes; no wrapping, clipping, fill, rectangle, color or page-count regression
occurs. Faces remain regular Calibri and Malgun Gothic. No emphasis, CJK text,
shape, border or dash occurs.

Full GT/output/diff pages, the matched crop, full-resolution header/footer
bands and all 18 current exact-ID crop triplets were inspected. All three
header baselines remain 1.599974pt high (#1731). Other small anchors/advances
remain under [#1874](https://github.com/developer0hye/office2pdf/issues/1874#issuecomment-6250743371).
The strict report classifies 12 header clusters under both #1731/#1874 and six
footer clusters under #1874. No renderer/exporter waiver is used. Raw layout
exits nonzero for these tracked findings; it is not an unqualified layout pass.
The rich-footer XLSX and DOCX header controls are byte-identical to baseline.

## Final memory and speed pass

Completed after scoped visual acceptance, with no further optimization code.
The accepted pre-optimization candidate is also the final candidate: scoped
measurements do not support a worthwhile low-complexity change to this helper.
Reported codegen medians are 1.863 / 1.879ms (main / candidate), about 0.22% of
CLI wall time. Elapsed ranges overlap; no cache or layout rewrite is justified.

| Input | Main elapsed median (range) | Candidate elapsed median (range) | Median peak RSS, main → candidate |
| --- | --- | --- | --- |
| reported | 0.8497s (0.8413–0.8590) | 0.8458s (0.8417–0.8602) | 94.609 → 94.469 MiB |
| rich-control | 1.2748s (1.2678–1.2897) | 1.2828s (1.2714–1.3069) | 152.109 → 151.109 MiB |

Hardware: Apple M2, Mac14,2, eight logical CPUs, 16GiB; macOS 26.6.2 build
25G83. Build: `cargo build --locked -p office2pdf-cli --profile ci`, separate
private targets. Fonts: installed Excel 16.113.4 DFonts plus the same system
font discovery, including Calibri/Malgun Gothic and Aptos for the rich control.
Seven measured sequential runs per binary/input follow one warm-up, with
alternating paired order, warm OS page cache and `RAYON_NUM_THREADS=2`; no task
build/test/conversion runs concurrently. Normal interactive host, not a
dedicated benchmark runner.

```sh
/usr/bin/time -l "$BINARY" "$INPUT" -o "$OUTPUT" \
  --font-path "$EXCEL_FONTS" --metrics
```

`EXCEL_FONTS` names Excel's installed DFonts directory. Inputs are the reported
workbook and `tests/fixtures/xlsx/issue_1181_fit_to_height.xlsx` (rich control).
Monotonic wall time includes CLI startup, fonts, conversion and output write;
Darwin maximum RSS is bytes, converted to MiB above, excluding builds/Office.
All measured PDFs match their audited/control hashes. No runtime code changed
after acceptance; full Rust validation and fresh strict visual evidence remain
applicable. These scoped results establish no product SLA or 100-page memory
guarantee.
