# M39 - Test Command Module Graph + Entrypoint Filtering

## What Was Added

Enabled multi-file module resolution for `sec4 test` entries and filtered runnable test entries to files that actually declare `fn main`.

Files:

1. `compiler/sec4-core/src/project.rs`
2. `compiler/sec4-core/src/lib.rs`
3. `compiler/sec4-cli/src/main.rs`
4. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. Core now exposes `resolve_modules_from_entry(source_root, entry_path)` so module graphs can be resolved outside manifest `src/` entry flow.
2. `sec4 test` now resolves module graphs per test entry from `tests/` root (imports inside tests are real, not file-isolated stubs).
3. `sec4 test` now executes only runnable entries (programs declaring `fn main`), so helper modules are importable without being executed as standalone tests.
4. `sec4 test` now fails deterministically when no runnable entrypoints are found and there are no static failures.
5. Missing-module diagnostics now include deterministic close-match suggestions (for example `did you mean: util`) to reduce rename/typo loops.
6. `sec4 test` summary now includes deterministic discovery observability:
   - `discovered=<count>`
   - `skipped_non_entry=<count>`

## Why

This closes a real usability gap for multi-file test projects: support modules under `tests/` can now be imported by runnable tests without being misclassified as standalone test executables.

## Validation

1. `cargo test -p sec4 --test commands test_command_supports_multi_file_test_modules`
2. `cargo test -p sec4 --test commands test_command_reports_missing_module_in_test_graph`
3. `cargo test -p sec4 --test commands test_command_fails_when_no_runnable_test_entrypoints_exist`
4. `cargo test -p sec4-core project::tests::`
5. `cargo test -p sec4 --test commands test_command_`
