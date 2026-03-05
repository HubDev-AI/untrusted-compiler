# M39 - LASM HTTP Runtime Pattern Override Order

## What Was Added

Adjusted pattern-route resolution order in LASM HTTP runtime.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`

## Behavior

1. Pattern routes now resolve in reverse registration order.
2. If multiple pattern routes match, the most recently registered pattern wins.
3. Exact routes still use deterministic overwrite semantics via hashmap insert.
4. Combined effect: both exact and pattern routes now follow last-write-wins behavior.

## Why

This aligns runtime behavior across route kinds and avoids stale-handler selection when route plans are re-registered.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`
2. `cargo test -p sec4 --test commands lasm_smoke_command_`
3. `cargo test -p sec4 --test commands`

New runtime coverage:

- `later_pattern_registration_overrides_earlier_pattern`
