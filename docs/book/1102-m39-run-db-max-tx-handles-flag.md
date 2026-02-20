# 1102 M39 Slice: Run `--db-max-tx-handles` Flag

This slice promotes LASM DB tx-handle capacity from env-only control to a first-class `sec4 run` flag.

## What changed

1. Added `sec4 run --db-max-tx-handles <N>` (`u64`) as an explicit CLI override.
2. Added deterministic CLI guards:
   - `--db-max-tx-handles` is LASM-only (`--backend lasm`).
   - `--db-max-tx-handles` must be `>= 1`.
3. Threaded the flag through LASM cluster worker forwarding so every spawned worker receives the same override.
4. Wired dynamic runtime state initialization to use CLI-over-env precedence:
   - explicit `--db-max-tx-handles` (highest)
   - `SEC4_RT_LASM_DB_MAX_TX_HANDLES`
   - default `256`
5. Kept deterministic platform-limit rejection for out-of-range conversion (`u64 -> usize`).

## Why

Tx-handle capacity is an operator tuning knob for long-lived LASM services. Keeping it env-only made per-run experiments and clustered rollouts less explicit.

A CLI flag gives deterministic, inspectable process configuration while preserving env fallback for existing automation.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_enforces_db_tx_handle_capacity_with_cli_flag_when_sqlite_adapter`
2. `cargo test -p sec4 --test commands db_max_tx_handles`
