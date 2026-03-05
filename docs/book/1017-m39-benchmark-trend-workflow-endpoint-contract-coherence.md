# 1017 M39 Slice: Benchmark Trend Workflow Endpoint Contract Coherence

## What It Is

This slice hardens benchmark-trend workflow contract validation by enforcing:

- endpoint-set coherence across:
  - `run_full_benchmark_suite.sh --endpoints ...`,
  - `check_regression_thresholds.sh --endpoint ...` steps,
  - `render_trend_note_entry.sh --endpoints ...`,
- one `--max-rss-kb` guard per threshold endpoint invocation.

## Why It Exists

Previous contract checks verified token presence, but they could still miss configuration drift such as:

- benchmark run scope and rendered trend-note endpoints diverging,
- missing RSS guard on one endpoint while another still had `--max-rss-kb`.

This slice makes drift fail fast at contract-test time instead of surfacing later in scheduled CI runs.

## How It Works Internally

1. Contract script extraction:
   - `scripts/test-benchmark-trend-workflow-contract.sh` now parses workflow flag values for:
     - `--endpoints`,
     - `--endpoint`,
     - `--max-rss-kb`.

2. Endpoint-set normalization:
   - endpoint lists are normalized (trimmed, lowercased, deduplicated, sorted),
   - all `--endpoints` declarations in workflow must match a single canonical set,
   - threshold endpoint set must match that same canonical set.

3. RSS guard cardinality:
   - number of `--max-rss-kb` flags must equal number of `--endpoint` threshold invocations.

4. Guard fixture coverage:
   - `scripts/test-benchmark-trend-workflow-contract-guard.sh` fixtures now include run + render endpoint declarations,
   - adds negative coverage for:
     - missing strict quality flag,
     - missing artifact upload,
     - missing per-endpoint RSS flag,
     - missing one threshold endpoint against multi-endpoint run scope.

## Inputs / Outputs and Constraints

Inputs:
- benchmark trend workflow YAML (`.github/workflows/benchmark-trend.yml`).

Outputs:
- contract pass/fail with endpoint-set and RSS-guard coherence enforcement.

Constraints:
- checks rely on explicit workflow flags (`--endpoints`, `--endpoint`, `--max-rss-kb`) being present in script-invocation lines.

## Failure Modes and Diagnostics

New deterministic failures include:

- endpoint-set mismatch between run/render/threshold definitions,
- RSS guard mismatch count (`--max-rss-kb` count differs from threshold endpoint count),
- missing run/render endpoint declarations required by contract.

## Example Usage

```bash
scripts/test-benchmark-trend-workflow-contract.sh
scripts/test-benchmark-trend-workflow-contract-guard.sh
```

Both commands must pass before claiming benchmark-trend workflow hardening changes are complete.

## Tradeoffs and Next Steps

Tradeoffs:
- contract parsing is text-based and assumes consistent CLI flag usage in workflow commands.

Next steps:
1. add a dedicated check that threshold step names align 1:1 with endpoint ids for clearer CI logs,
2. extend contract checks for impl-set coherence (`--impls`) if trend workflow scope expands beyond `node`.
