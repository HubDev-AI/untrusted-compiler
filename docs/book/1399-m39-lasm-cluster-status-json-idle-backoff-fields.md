# M39: LASM Cluster Status JSON Idle Backoff Fields

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Extended LASM cluster status snapshot/payload with:
  - `relayIdleSpinThreshold`
  - `relayIdleSleepMicros`
- `spawn_lasm_cluster_status_writer` now resolves idle backoff env controls once per writer thread and stamps values into each status snapshot.
- Snapshot equality includes both fields so unchanged-snapshot suppression still behaves deterministically.
- Updated command integration coverage in `compiler/sec4-cli/tests/commands.rs` to assert env-overridden values in emitted status JSON.

## Why

Idle backoff env tuning was added in the previous slice, but status JSON did not expose the effective runtime values. Operators could tune values but could not confirm live configuration via status artifacts. This slice closes that observability gap.

## Result

- Cluster status JSON now exposes effective idle backoff configuration alongside existing relay/queue/autoscale telemetry.
- Operators can verify active tuning values without shelling into process env or rebuilding.
- Deterministic `updatedAtMs` suppression contract remains intact when snapshots are unchanged.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_cluster_status_json.rs compiler/sec4-cli/src/lasm_cluster_status_writer.rs compiler/sec4-cli/tests/commands.rs`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
