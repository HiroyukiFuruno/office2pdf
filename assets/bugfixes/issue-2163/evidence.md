# First-only header repetition

`compare.jpg` shows page 2 of the public upstream preferences form: supplied
Microsoft Word reference on the left, unchanged main-runtime output on the right.
Both panels are full-page renders at 150 DPI. No source document or PDF is copied.

Source: `sverrejb/docxide-pdf` at `adb8c2b82baad8deb2973d1d059bfb7513b8e1fe`,
`tests/fixtures/scraped/italian_teacher_hiring_preferences/input.docx`.
Input SHA256: `921b15418f66a34f10263e0c6bb059f70c6f53f2677b83ab396ef75e3ce7154d`.
Native PDF SHA256: `4550660847eb720a84894956a921acda37e1943fc262e89199ff57ac9ca53a22`.
Output PDF SHA256: `cb33c9274909b9d40bd63907247d4a76847bb5084d49fc03935184a0cef7f08a`.

The section declares `w:titlePg` and a first header, with no default header.
Main promotes the first story to the default and repeats its logo, emblem and
script letterhead across continuation text. Native page 2 has none of that art.
The footer path contains the corresponding promotion rule.

Both pages were reviewed using geometry/text/render reports, full pages, diffs
and all 35 baseline matched shift crops at native resolution. A separate local
selection control also had its 35 matched crops inspected. This records a defect;
conversion code is unchanged, and this does not declare either page correct.
Separate findings include the cover body title 67.12092pt too high, displaced
body/table content, a first-page footer 13.525pt too low, and checkbox outlines
at 0.75pt rather than the theme/native 2pt. Fine placement and image-sampling
residuals also remain. The PR audit links their open tracking issues.
