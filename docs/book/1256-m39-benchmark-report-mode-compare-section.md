# 1256 M39 Slice: Benchmark Report Mode-Compare Section

This slice surfaces LASM proxy-vs-fixed mode comparison results directly in the benchmark markdown report.

## What changed

1. `publish_report.sh` input contract:
   - Added optional seventh argument: `mode_compare.json`.
   - Added deterministic validation when provided:
     - file exists,
     - `comparison.recommendedMode` is present.
2. Report output:
   - Added `## LASM Mode Comparison` section with:
     - recommended mode,
     - proxy/fixed pass status,
     - proxy/fixed req/s,
     - req/s delta and gain percent vs proxy,
     - proxy/fixed p99 in ms,
     - peak RSS delta in KB.
3. Full-suite forwarding:
   - `run_full_benchmark_suite.sh` now forwards `results/summaries/sec4-lasm-cluster-mode-compare.json` into `publish_report.sh` when `--include-lasm-mode-compare` is enabled.
4. Test fixtures/contracts:
   - Added fixture: `benchmark-suite/scripts/testdata/sample-mode-compare.json`.
   - Updated `test_publish_report.sh` to verify mode-compare section rendering.
   - Updated `test_run_full_benchmark_suite.sh` to verify publish command includes mode-compare artifact when mode-compare phase is enabled.

## Why

Mode comparison had already been emitted as JSON, but operators still had to inspect separate artifacts manually. Embedding a concise summary in the final benchmark report keeps one canonical operator-facing output.

## Validation

1. `bash -n benchmark-suite/scripts/publish_report.sh benchmark-suite/scripts/run_full_benchmark_suite.sh benchmark-suite/scripts/test_publish_report.sh benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
2. `benchmark-suite/scripts/test_publish_report.sh`
3. `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
4. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
