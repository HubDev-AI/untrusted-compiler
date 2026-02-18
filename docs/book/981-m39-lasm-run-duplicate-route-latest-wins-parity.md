# M39: LASM Run Duplicate-Route Latest-Wins Parity

## Why

`run --backend lasm` route planning deduplicated routes with first-registration-wins behavior.

That diverged from LASM runtime registration semantics, where latest registration for the same method/path should win.

## What Changed

1. Replaced first-seen deduplication in LASM run route planning with deterministic latest-registration-wins consolidation.
2. Preserved deterministic plan ordering by sorting surviving registrations by last-seen index.
3. Added LASM oneshot integration coverage proving duplicate route method/path pairs resolve to the latest handler.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_uses_latest_duplicate_route_registration`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_applies_helper_side_effect_middleware_when_router_arg_is_not_first`
