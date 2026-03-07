# M39 Final Alpha Closeout: Proof Bundle And Readiness

Date: `2026-03-07`

## What closed the alpha

The alpha closed on one proof chain and one benchmark publication family:

1. proof bundle:
   - `scripts/run-final-alpha-proof-bundle.sh --artifacts-dir build/release-alpha-gate`
2. benchmark publication:
   - `benchmark-suite/results/workbench-benchmark-report.md`
   - `benchmark-suite/results/workbench-benchmark-report.html`
   - `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
   - `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
   - `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`

The final decision record is:

- `docs/plans/2026-03-07-final-alpha-readiness-summary.md`

## Why the benchmark family needed reconciliation

Two benchmark families existed in the branch:

- `workbench-benchmark-*`
- `workbench-full-benchmark-*`

Only the first one was accepted as the alpha-closeout publication family.

Reason:

- `workbench-benchmark-*` is the fresh six-endpoint, four-implementation publication set.
- `workbench-full-benchmark-*` is a narrower tuning/exploration artifact family and still records failed suite phases.

## Final blockers that were fixed

Two deterministic closeout blockers had to be removed before the proof chain could pass:

1. oneshot CORS test race
   - fixed by switching the affected test reads from EOF-based `read_to_string(...)` to the structured `read_http_response(...)` helper
2. language-server workspace scan regression
   - fixed by stopping unopened-workspace fallback for URIs outside a real `sec4.toml` project root

## What remains after alpha

Alpha closure does not mean all performance work is done.

The explicit next queue is:

1. runtime/scaling tuning on the canonical Postgres workbench workload
2. benchmark-runner cleanup so run-audit and publication surfaces converge more cleanly
3. `wb-tasks-list` saturation-path work

## Outcome

The project now has:

- a real no-stub alpha proof chain
- a canonical Postgres-backed LASM app
- benchmark publication against `node`, `go`, and `rust`
- a final readiness summary that says the alpha is ready
