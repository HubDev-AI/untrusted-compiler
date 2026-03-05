# M39: LASM Saturation Matrix/Bundle Build-Profile + Multi-Sample Wiring

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh`:
  - added `--build-profile <debug|release>` and `--samples <n>`,
  - forwards both controls to every per-step probe and to the optional recommended-step verify probe,
  - dry-run verify command preview now includes build-profile/sample controls,
  - added deterministic input validation for invalid build-profile/samples values.
- Updated `benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh`:
  - added `--build-profile <debug|release>` and `--samples <n>`,
  - forwards controls into matrix execution, preserving one consistent probe configuration across the full bundle path.
- Updated script tests:
  - `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`,
  - `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`,
  - both now validate build-profile/sample passthrough and invalid-input diagnostics.

## Why

Capacity probe controls were already stabilized in the base probe and mode-compare wrapper, but saturation orchestration still used implicit defaults. This made matrix/bundle tuning runs less reproducible than direct probes.

## Result

- Saturation tuning pipelines now support explicit and repeatable profile/sample control end-to-end:
  - matrix: `--build-profile release --samples 3`,
  - bundle: same controls forwarded to matrix and verify probe.
- Dry-run command plans and tests now lock these controls so CI/operator workflows remain deterministic.

## Validation

- `bash -n benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh`
- `bash -n benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh`
- `bash -n benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`
- `bash -n benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`
