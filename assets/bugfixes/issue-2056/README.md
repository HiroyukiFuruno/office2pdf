# Issue 2056 visual evidence

This directory holds the evidence for the third #2056 fix: an empty header
paragraph reserves one line of its paragraph mark above the body. The
wrapped-line evidence from #2057 remains at merge commit
`d65372fb7f0172edc1c8940e947c5cc068743bf6`, and the exact-line evidence from
#2219 at `6a4b7992b1a592644e713b1443253bba6f551713`.

Source, not committed: docxide-pdf
[`tests/fixtures/scraped/construction_quality_control_spec`](https://github.com/sverrejb/docxide-pdf/tree/fe7b8beafa7612cc89ad62a7d27b2c6141c4d6ed/tests/fixtures/scraped/construction_quality_control_spec)
at `fe7b8beafa7612cc89ad62a7d27b2c6141c4d6ed`, a public three-page
specification. Its `reference.pdf` reports `Creator: Microsoft Word`, US
Letter, three pages, and `check_gt_integrity.py --source` passes. The tracked
regressions build their own synthetic packages instead.

Its header is three Calibri 11pt lines and an empty paragraph whose mark
resolves to Times New Roman 12pt (`Header` style, `Normal` 12pt,
`w:rPrDefault` Times New Roman). `w:header` is 36pt and `w:top` 64.8pt.

SHA-256:

- Source: `b2c0d12a5dd997088ba597e35f4b5665876cffcc2088db3cc448c2bd06153e06`
- Word reference: `5f75e91c24dccfcb63e7d6a987a48ac975891d92cfb3230456839b0b0b6df610`
- Before output (main `3e946f31`): `dc7c63616e9638e67ece9fdac86a91ef2b1343dd0d3ba1c75241bac9403a2e32`
- After output: `9177cb6a0c0407a843b583ca8507ac73ee39b1745b25847b9bdddb9c17155223`

Both CLIs were built with `cargo build --locked --profile ci -p
office2pdf-cli` in separate target directories; the output is Typst 0.15.1.
`layout-audit.json` comes from `compare_layout.py --json --audit --fine-shift
0.5` over all three pages. Each `render-clusters-page-<page>.json` comes from
`compare_render.py --dpi 150 --fine-shift 0.5 --strict-clusters`. The JPEGs
are page 1 as pdftoppm 150 DPI renders, saved as progressive quality 86
without metadata.

Result, as word-box deltas from `pdftotext -bbox` (the faces are
metric-compatible, so they track the baselines):

| line | before | after |
|---|---:|---:|
| header lines 1-3 | -3.29 to -3.47pt | -0.05 to +0.12pt |
| `SECTION 01 40 00` (first body line) | -25.26pt | +0.03pt |
| `PART 1 GENERAL` | -25.24pt | +0.04pt |

Main measured no header height once a paragraph was empty, so the body sat on
the 64.8pt margin and overprinted the third header line. The band also had no
room for the header's ascent shift, which lifted the header lines 3.4pt. After
the fix every page breaks where Word's does, and `compare_layout.py` findings
fall from 212 to 28. The text layer differs only as listed below.

Remaining on all three pages, present on main too:

- `1.%1` labels print `1` … `9` where Word prints `1.1` … `1.9` (#2221).
- The footer's first line sits 33.28pt above Word's and overprints the last
  body line on pages 1 and 2 (#2222). In text extraction it merges into that
  line.
- Page 3's `PREPARATION:` starts 14.40pt left of Word's default stop (#2223).
- Sub-material residuals, recorded in #1874: body words within 0.27pt
  vertically, drifting up to 1.77pt horizontally along long tracked lines;
  `DMS 2023 Edition` 0.81pt left.

Other Word references checked, as first-body-line baseline dy (main, then
after): `centrifugal_water_chillers` page 1 (nine pages, 11pt empty mark),
-16.81pt to +0.13pt; `federal_procurement_terms` page 1 (one empty header
paragraph, `w:top` = `w:header`), -13.56pt to -0.13pt;
`radiographer_interventional_job_desc` page 3 (two empty paragraphs),
-26.78pt to +0.02pt.
