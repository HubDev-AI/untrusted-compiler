# 1505 M39 Slice: Alpha Postgres Suite Reset-Flag Make Wiring

## What changed

1. Added `BENCH_ALPHA_POSTGRES_RESET_BETWEEN_PHASES` Make variable in `benchmark-suite/Makefile`.
2. Wired the variable into alpha-suite targets:
   - `bench-alpha-postgres-suite`
   - `bench-alpha-postgres-suite-dry`
   - `bench-alpha-postgres-suite-local`
   - `bench-alpha-postgres-suite-local-dry`
3. Updated benchmark README usage notes to surface the Make-variable toggle.

## Why

The reset behavior existed at script-level, but one-command operator loops usually run through Make. Wiring the flag through Make keeps local/CI invocation paths consistent without requiring manual script argument edits.

## Validation

1. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
2. `make -C benchmark-suite bench-alpha-postgres-suite-local-dry BENCH_ALPHA_POSTGRES_RESET_BETWEEN_PHASES=true`
