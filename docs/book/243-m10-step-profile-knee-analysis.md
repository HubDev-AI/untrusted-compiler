# 243 M10 Slice: Step-Profile Knee Analysis

This chapter documents automated knee/saturation analysis for step-load benchmark outputs.

## What it is

Updated:
- `benchmark-suite/scripts/analyze_step_profile.sh`
- `benchmark-suite/scripts/test_analyze_step_profile.sh`
- `benchmark-suite/scripts/testdata/sample-step-summary-decode.json`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- added analyzer for `*-step.json` artifacts,
- computes achieved ratio per step (`requestsPerSec / targetRps`),
- detects first knee point below configurable threshold (default `0.9`),
- extracts p99 spread bounds (`p99MinMs`, `p99MaxMs`),
- added Make target `analyze-step-profile`.

## Why it exists

Step-load runs generate many summaries. Without an analyzer, spotting saturation behavior is manual and inconsistent. This slice adds deterministic knee detection and summary metrics for repeatable interpretation.

## How it works internally

1. Load step summary artifact and validate non-empty `steps`.
2. Normalize per-step metrics:
   - target RPS
   - achieved RPS
   - achieved ratio
   - parsed p99 milliseconds
3. Detect first step where achieved ratio falls below threshold.
4. Emit analysis JSON with step rows and summary metrics.

## Inputs, outputs, and constraints

- Inputs:
  - `analyze_step_profile.sh <step_summary.json> <out_analysis.json> [knee_ratio_threshold]`
- Outputs:
  - JSON analysis artifact with `kneeDetected`, `kneeAtTargetRps`, and p99 bounds.
- Constraints:
  - step summary must include at least one step record with target/observed rates.

## Failure modes and diagnostics

- missing step summary input -> explicit file-not-found error.
- empty/invalid step summary -> explicit validation error.

## Example usage

```bash
make -C benchmark-suite analyze-step-profile IMPL=sec4 ENDPOINT=decode
```

Custom threshold:

```bash
benchmark-suite/scripts/analyze_step_profile.sh benchmark-suite/results/summaries/sec4-decode-step.json benchmark-suite/results/summaries/sec4-decode-step-analysis.json 0.85
```

## Tradeoffs and next steps

- Tradeoff:
  - knee detection uses a simple threshold heuristic rather than curve fitting.
- Next:
  - integrate step-analysis summary into markdown publish output when step artifacts are present.
