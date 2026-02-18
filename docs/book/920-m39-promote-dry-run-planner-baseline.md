# M39 - Promote Dry-Run Planner Baseline

## What Was Added

Introduced baseline promotion planning command surface for browser-to-server transition:

- `sec4 promote --from browser --to server --dry-run [--path <project>]`

Files:

1. `compiler/sec4-cli/src/main.rs`
2. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. `promote` currently supports only dry-run mode; apply mode is rejected deterministically with usage-style guidance.
2. Supported route is explicitly constrained to `browser -> server`; unsupported direction pairs fail deterministically.
3. Dry-run emits deterministic JSON plan artifact containing:
   - `changedBindings`
   - `generatedFiles`
   - `scannedSources`
   - `preconditions`
   - `ready` flag
4. Planner records compile/validation diagnostics as deterministic promotion preconditions (`DIAG.<code>`).
5. Planner emits deterministic warning when no `localdb` references are present (`PROMOTE.P9303`) to highlight potential storage rewrite no-op scenarios.
6. Blocking preconditions still emit the plan artifact and return non-zero exit for CI safety.

## Why

This establishes the S3 promotion planning contract without changing source trees yet, enabling deterministic preview and gating workflows before S4 apply/rewrite work.

## Validation

1. `cargo test -p sec4 --test commands promote_`
2. `cargo test -p sec4 --test commands`

New coverage:

- `promote_dry_run_emits_deterministic_plan_for_valid_project`
- `promote_fails_without_dry_run`
- `promote_dry_run_reports_blocking_preconditions_for_invalid_project`
