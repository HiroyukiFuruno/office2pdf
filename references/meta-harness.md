# Meta-Harness

- Paper: Lee et al., "Meta-Harness: End-to-End Optimization of Model Harnesses",
  arXiv:2603.28052v1 (2026-03-30), https://arxiv.org/abs/2603.28052
- Code: https://github.com/stanford-iris-lab/meta-harness-tbench2-artifact

A coding-agent proposer improves an LLM harness by reading every prior candidate's
source, scores and execution traces from a filesystem with grep/cat, instead of
scalar scores or summaries. Its Table 3 ablation (text classification) reports
median accuracy 50.0 with raw traces versus 34.9 with scores plus summaries;
Appendix D advises machine-readable, consistently named logs and a small query CLI.

Applied to `scripts/issue_loop.sh`:

- `scripts/issue_loop_history.py` keeps each run's prompt, report, outcome and full
  transcript under `target/issue-loop-logs/runs/`, with an append-only index.
- The next attempt at an issue gets its earlier attempts' outcomes and transcript
  paths, plus a preflight note when merged pull requests already cite it.
- `--reflect-every N` runs the outer loop advisorily: the proposer may only open a
  pull request against the loop's own files, because issue outcomes are too coarse
  a score to accept prompt changes unreviewed.
