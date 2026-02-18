# M39 - Promote Dry-Run Planner Baseline

## What Was Added

Introduced baseline promotion planning command surface for browser-to-server transition:

- `sec4 promote --from browser --to server --dry-run [--path <project>]`

Files:

1. `compiler/sec4-cli/src/main.rs`
2. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. S3 baseline introduces deterministic dry-run planning mode (`--dry-run`) for promotion prechecks.
2. Dry-run supports optional artifact output path (`--out <path>`) and writes byte-equivalent JSON to stdout + artifact file.
3. Supported route is explicitly constrained to `browser -> server`; unsupported direction pairs fail deterministically.
4. Dry-run emits deterministic JSON plan artifact containing:
   - `changedBindings`
   - `generatedFiles`
   - `scannedSources`
   - `preconditions`
   - `ready` flag
5. Planner records compile/validation diagnostics as deterministic promotion preconditions (`DIAG.<code>`).
6. Planner emits deterministic warning when no `localdb` references are present (`PROMOTE.P9303`) to highlight potential storage rewrite no-op scenarios.
7. Blocking preconditions still emit the plan artifact and return non-zero exit for CI safety.

## Why

This establishes the S3 promotion planning contract and deterministic preview/gating workflow. S4 apply/rewrite behavior is documented separately in chapter `921`.

## Validation

1. `cargo test -p sec4 --test commands promote_`
2. `cargo test -p sec4 --test commands`

New coverage:

- `promote_dry_run_emits_deterministic_plan_for_valid_project`
- `promote_dry_run_writes_plan_artifact_when_out_is_provided`
- `promote_rejects_unsupported_route_pair`
- `promote_dry_run_reports_blocking_preconditions_for_invalid_project`
