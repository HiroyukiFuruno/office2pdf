# Agent Rules

This is the canonical instruction source for all coding agents.

- Communicate and work in English.
- Read and follow `METHODOLOGY.md` before any task and `PRD.md`, if present, before development.
- Prefer native Rust and the standard library; justify third-party dependencies.
- Keep `AGENTS.md`, `CLAUDE.md`, and user documentation concise. After code changes or merges, update `README.md` if usage, setup, architecture, or APIs changed.
- Complete, validate, and commit each planned task before starting the next.
- **Before every commit, delegate a read-only documentation freshness audit and wait for `PASS:`.** Codex uses `documentation_freshness_reviewer`; Claude Code uses `documentation-freshness-reviewer`. Check existing documentation, examples, and comments against current code/configuration; fix stale or unverified claims in the same commit.

## Testing (TDD)

- For every feature or fix, write a failing behavioral test, implement the general rule, then refactor. Use realistic inputs and triangulation; never hardcode test cases or weaken tests to pass.
- Keep tests isolated and use default parallelism. Investigate slow suites; run fast unit tests before slow integration tests.
- Use the tracked `Cargo.lock`: pass `--locked` to build, test, and evidence commands. Update dependencies explicitly and review the lockfile diff.
- Run the full Rust suite with `cargo test --locked --workspace --profile ci`.
- Skip runtime tests for changes without runtime impact (documentation, comments, CI configuration); runtime configuration changes still require tests. Run applicable documentation/CI contract checks. Markdown-, `assets/`-, and `references/`-only PRs skip Rust CI jobs; workflow edits do not.

## Test Fixture Storage

- Do not introduce Git LFS without running `lfs-cost-advisor` and following its recommendation, including before adding `lfs: true` or `git lfs pull` to CI.
- Track every fixture referenced outside `tests/bulk_conversion_baseline.json`, including fixtures used by default tests, golden mocks, and visual audits. Bulk-only fixtures come from the release archive via `.github/actions/bulk-fixtures`.
- Files under `tests/fixtures/*/libreoffice/*` and `tests/fixtures/*/poi/*` are ignored. Before force-adding one, confirm a non-ignored test needs it.
- To replace the bulk corpus, use `scripts/download-third-party-fixtures.sh`, publish a new `fixtures-vN` release, and update `tag`, `sha256`, and `min-files` in `.github/actions/bulk-fixtures/action.yml`. On macOS, build the archive with `COPYFILE_DISABLE=1 tar czf` to exclude AppleDouble members.
- Declare every `mc:Ignorable` namespace prefix in scope; `fixture_package_validity.rs` checks authored packages.

## Code Style

- Use descriptive names, predicate booleans, and action/target function names. Annotate non-obvious types and use newtypes for domain units.
- Comments explain reasons and constraints. Mark limitations with `TODO(reason)` or `FIXME(reason)` and remove stale comments.
- Use `tracing` at key decisions, transitions, and external calls, with relevant context, elapsed time, and outcome. Prefer `tracing::instrument`; avoid hot-path noise and never log credentials or personal data.

## Reference Projects

- Before design decisions, check `references/INDEX.md`. If no relevant reference exists, research maintained open-source implementations across languages.
- Add useful discoveries to the index and a detail file under 50 lines; cite references that informed the implementation.
- If a dependency bug or limitation breaks conversion, fix and test it upstream and open a PR following that project's conventions.

## Confidentiality

- Never publish names, paths, companies, people, titles, or other content from classified fixtures in commits, PRs, issues, or public text. Use generic descriptions such as "internal test document" or "ground truth PDF".
- Obtain user confirmation before publishing images from classified fixtures.

## Git and Pull Requests

- Use `developer0hye` for this repository's GitHub operations. Before writes, verify `gh auth status` and `gh api user --jq .login`; switch accounts and reverify if necessary. Do not use connectors with an unverified acting identity.
- Before committing, verify local `user.name=Yonghye Kwon` and `user.email=developer.0hye@gmail.com`. Use `git commit -s`; author and sign-off must match.
- All changes go through PRs. Use one focused `<type>/<short-description>` branch in a **git worktree**, based on current `main`. Work and push there; never switch branches or commit directly in the main worktree.

### PR Merge Procedure

1. Give the PR a clear description of the problem, change, and validation. Cross-reference related issues with `Related: #N`; avoid auto-close keywords unless instructed.
2. Resolve conflicts with current `main`, then wait for passing CI using `gh pr checks <number> --watch`. Do not merge failing checks.
3. Review `gh pr diff <number>` for correctness, debug code, hardcoded paths, credentials, and unused imports.
4. Merge with `gh pr merge <number> --merge`; never use `--delete-branch` while its worktree exists.
5. Sync the main worktree and verify CI on the exact merge commit before cleanup.
6. Check worktree status, merge ancestry, and open PRs. Preserve unrelated changes, active work, unmerged commits, required evidence, and shared build data.
7. In the same turn, remove the completed worktree with `git worktree remove`, delete its local branch with `git branch -d` and its remote branch with `git push origin --delete`, then prune stale records. Do not force-delete unfinished work; no additional confirmation is needed for verified cleanup.

## MSRV Policy

- `rust-version` must be a stable release at least six months old and meet the edition floor (Rust 1.85 for edition 2024). Check release dates at https://releases.rs/.
- Bump only when required by language features or dependencies. Verify code and dependency compatibility with the target, and update the `Check MSRV` toolchain in `.github/workflows/ci.yml`.

## Visual Comparison Workflow

Read [the evidence contract](assets/bugfixes/README.md) and use
[the PR template](.github/pull_request_template.md) for every visual fix or defect
report. The following rules govern the audit; report schemas and exporter-exception
requirements are documented in that contract.

### Scope and provenance

- Audit **every page of every reported file**. Before changing code, record deviations in page count/order, element presence, position/size, rotation/flip, fill, stroke/border/dashes, shape outline, text content, font family/weight/style/color, alignment, spacing, and clipping/overflow.
- Verify GT provenance with `pdfinfo`; an attached PDF may be converter output. For user-supplied files on macOS, export from the matching Microsoft app when available. Otherwise generate and identify a LibreOffice reference; never assume it reproduces native Office behavior.
- Run `python3 scripts/check_gt_integrity.py GT.pdf [--source FILE.xlsx]` before deriving measurements. Read source XML and identify the element/attribute responsible before attributing a defect; reference-side extraction or font substitutions are not automatically converter bugs.
- Use `mutool` for geometry (`brew install mupdf-tools` on macOS). Do not quote APPROXIMATE fallback measurements in issues or PRs.

### Required comparisons

Run geometry first, then text, color/pixels, and direct image inspection:

```sh
python3 scripts/compare_layout.py GT.pdf OUTPUT.pdf --json --audit --fine-shift PT
python3 scripts/compare_text_layer.py GT.pdf OUTPUT.pdf
python3 scripts/compare_render.py GT.pdf OUTPUT.pdf --page N --dpi 150 \
  --fine-shift PT --artifacts-dir ARTIFACT_DIR --cluster-report REPORT.json
```

- Run the render comparison for every compared page with a separate artifact directory. Read each tool's interpretation and inspect all emitted full GT/output pages, pixel diffs, and matched region crops with the acting agent's image vision.
- Render at **150 DPI or higher**; inspect text, lines, and decorations in matched full-resolution crops. Inventory elements at most 1pt wide, dash patterns, and bold/italic/underlined runs, including CJK. Record concrete observations in `Model vision findings`; numeric reports alone are insufficient.
- Choose `--fine-shift PT` using the materiality rule below and keep it at least the trace noise floor. Set `--noise-floor` explicitly when needed (0.12pt for Word, 0.5pt for Excel).
- Fix or track every page-count, missing/extra/reflow/changed-wrap, painted-text visibility, visible-fill occlusion, rectangle geometry, and fine/large text-instance shift finding. Safe split/join topology is informational; recovered fragments still require position and visibility checks.
- Geometry shifts are not antialiasing. Assess histogram **color shift**, not bin intersection, and never infer correctness from pixel counts alone. For text changes, inspect both codepoint classes and normalized content; identical pixels do not prove searchable text is correct.

### Cluster dispositions and materiality

- Inspect current crops and assign every cluster ID to an accepted renderer class, an open issue, or a verified exact-ID reference-exporter difference. Accepted classes: glyph-edge rasterization, photo resampling, gradient rasterization, shape-edge antialiasing.
- Re-derive dispositions on every audit; never inherit them by stable ID or bounding box. Page/region/bbox-wide approvals are invalid. Keep historical reports frozen and record re-audits separately; an absent cluster has no current disposition. `renderer_observations` never dispositions a cluster.
- Rerun each page with `--cluster-dispositions PATH --strict-clusters` and require a passing report. For exporter exceptions, also pass `--reference-differences PATH` and follow the exact-ID/provenance contract; do not waive unrelated findings.
- **One issue and PR per root cause; fix sequentially.** File separate issues for independent material deviations, with the identifying measurements.
- **Materiality:** a deviation is material if a reader notices it without an overlay. Changed wrapping/pagination, missing/overlapping/clipped text, or a wrong face/color are always material. Calibrate reference placement around a couple of points, internal alignment around a device pixel, and stroke/size differences proportionally (roughly a quarter); these are guides, not fixed gates. Record sub-material findings and their measurements under #1874 instead of creating separate issues.

### Evidence and completion

- Keep `gt.jpg`, `before.jpg`, `after.jpg`, `layout-audit.json`, and one `render-clusters-page-<page>.json` per compared page under `assets/bugfixes/issue-<number>/`. Use the same fixture, pages, renderer, and resolution. Capture `before.jpg` before the fix; generate current after images and reports from the same GT/output PDFs.
- A raster-changing fix must change `after.jpg`, `layout-audit.json`, and every compared page's strict report. List all reports and the matching fine threshold in the PR; classify remaining findings in `Visual audit` and matching `Remaining:` rows.
- For a text-layer-only fix, regenerate and verify `after.jpg` without annotations. An identical image need not appear in the diff. Declare `Text-layer-only: Yes` and `Pixel delta: 0`, change the layout report, and demonstrate resolved missing/extra searchable text; retain both images for exact decoded-pixel verification.
- Publish progressive JPEGs at quality 86, preserving dimensions and legibility. Strip metadata **before** restoring density (`-strip -density 150 -units PixelsPerInch`); never upscale to meet the DPI requirement.
- When filing a visual issue, commit `compare.jpg` with GT left and current output right, from the same page/resolution, and embed a commit-pinned raw URL in the issue.
- Close an issue only after a fresh comparison proves its defect gone and every remaining material deviation has an open issue or verified exact-ID exporter disposition. Re-audit every posted after image and reference all remaining deviations in the PR.
- After each merge, repeat the full reported-file audit, file newly exposed defects, and fix them in turn. Report a file done only when a full pass finds nothing new.

### Native probes and artifacts

- Settle unclear layout rules with one-factor XML probes using `scripts/probe_harness.py`; read its docstring for the spec and backend options. Require the no-patch re-zip control to match the base; a failed control or differ invalidates the probe.
- Stage native Office input/output inside the **driven app's own container on the internal disk** (`~/Library/Containers/com.microsoft.Word`, `.Powerpoint`, or `.Excel`). Other locations can trigger per-file access dialogs; do not stage exports under `/Volumes`. Copy evidence out afterward.
- Generate internal comparison artifacts with `cargo test --locked -p office2pdf --test artifact_generator -- --ignored --nocapture`. Use the generated report to locate pages, then apply the same full-page audit and confidentiality rules.

## Unattended Issue Loop

- `scripts/issue_loop.sh` selects the oldest eligible open issue and starts a fresh Claude process per attempt. Repository-specific instructions live in `scripts/issue_loop_prompt.md`.
- Trial one attempt with `scripts/issue_loop.sh --once --max-issues 1 --expect-user developer0hye`. `--once` exits when the queue is empty; `--max-issues` counts attempts, not resolved issues.
- Label umbrella issues `autofix-skip`; `autofix-blocked` excludes stalled issues. Use `scripts/issue_loop_watch.py` for live activity and `--help` for runner options. A `target/issue-loop-logs/STOP` file requests exit between iterations when using the default log directory.
- In headless sessions, wait for CI in the foreground and complete the PR lifecycle before ending the turn. A runner progress message does not replace the completion checks above.

## Release Procedure

Follow `RELEASING.md` end to end, including identity checks, the version PR,
workflow dispatch, publication verification, and cleanup. A version bump or
GitHub Release alone is not completion.
