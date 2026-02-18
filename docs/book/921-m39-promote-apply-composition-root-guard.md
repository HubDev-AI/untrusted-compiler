# M39 - Promote Apply Composition-Root Guard

## What Was Added

Extended `sec4 promote` from dry-run-only planner into apply mode with deterministic scaffold generation and composition-root-only rewrite behavior.

Files:

1. `compiler/sec4-cli/src/main.rs`
2. `compiler/sec4-cli/tests/commands.rs`

## Behavior

1. `sec4 promote --from browser --to server` (without `--dry-run`) now executes apply mode when preconditions are non-blocking.
2. Apply mode enforces composition-root-only rewrite guard:
   - rewrites `localdb.` tokens only in `src/main.ut`
   - preserves non-root module files unchanged
   - reports skipped non-root references under `guardedSkippedReferences`
3. Apply mode generates deterministic scaffold files:
   - `server/src/main.ut`
   - `server/src/repo/db_repo.ut`
   - `server/db/schema.sql`
   - `server/deploy/sec4.server.toml`
4. Apply mode writes deterministic report artifact:
   - `server/reports/promote-plan.json`
5. Unsupported route pairs (for example `server -> browser`) still fail deterministically with usage-style exit code.

## Why

This delivers S4 baseline promotion mechanics while preserving safety boundaries: rewrite only the composition root and keep all non-root module rewrites explicit instead of hidden side effects.

## Validation

1. `cargo test -p sec4 --test commands promote_`
2. `cargo test -p sec4 --test commands`

New/updated coverage:

- `promote_apply_rewrites_composition_root_and_generates_scaffold`
- `promote_rejects_unsupported_route_pair`
