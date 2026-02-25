# 1517 M39 Slice: Postgres Connect-Timeout DSN Fragment-Safe Rewrite

## What changed

1. Updated `compiler/sec4-cli/src/lasm_db_adapter_state.rs` DSN timeout rewrite path:
   - added `split_lasm_postgres_dsn_fragment(...)`,
   - `build_lasm_postgres_connect_dsn(...)` now injects timeout into the base DSN and reattaches fragment suffix.
2. Added unit coverage for:
   - fragment-bearing URL DSNs with and without existing query strings,
   - existing timeout preservation on fragment-bearing DSNs,
   - fragment split helper behavior.

## Why

Previous timeout injection appended to full URL strings. For DSNs containing `#fragment`, this could place `connect_timeout` after the fragment marker, where it is ignored by connection parsing.

## Validation

1. `cargo test -p sec4 --bin sec4 postgres_connect_dsn_injects_timeout_before_fragment -- --nocapture`
2. `cargo test -p sec4 --bin sec4 split_postgres_dsn_fragment_handles_fragment_and_non_fragment_cases -- --nocapture`
