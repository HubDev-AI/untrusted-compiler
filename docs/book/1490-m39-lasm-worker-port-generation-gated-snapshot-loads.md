# M39 Slice: LASM Worker-Port Generation-Gated Snapshot Loads

Date: 2026-02-23
Milestone: M39-S2B (LASM async backend bootstrap, scale hardening lane)

## What changed

- Added shared worker-port generation counter wiring across LASM cluster runtime components.
- `refresh_lasm_cluster_worker_ports_snapshot_if_changed(...)` now returns a change flag.
- Autoscale loop increments worker-port generation only when snapshot content actually changes.
- Relay worker loop now checks generation first and only performs `ArcSwap` worker-port snapshot reload when generation changed.
- Status writer loop now also checks generation first before loading worker-port snapshots.

## Why

- Relay/status loops were repeatedly loading worker-port snapshots even when cluster topology was unchanged.
- Generation gating avoids unnecessary atomic snapshot loads and pointer comparisons in tight loops.

## Behavioral contract

- No behavior change in routing/autoscale semantics.
- Snapshot reloads stay deterministic and still happen immediately after worker-port topology changes.
- Runtime now avoids no-op snapshot load work when topology is stable.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_`

