# 1061 M39 Slice: LASM Cluster Status JSON Telemetry

This slice adds optional operator-facing status telemetry output for LASM cluster mode.

## What changed

1. Added new run flag:
   - `sec4 run --cluster-status-json <path>`
2. Flag contract:
   - supported only with `--backend lasm`,
   - requires cluster mode (`--instances > 1` or `--autoscale-max-instances > 1`),
   - rejected for fixed reuse-port mode (`instances == autoscale-max-instances`) because there is no cluster front-proxy relay path in that mode.
3. In proxy cluster mode, when the flag is set, front process starts a status-writer thread that periodically writes JSON snapshots:
   - `mode`, `updatedAtMs`, `listenPort`,
   - `minInstances`, `maxInstances`,
   - `workerCount`, `workerPorts`,
   - `activeConnections`,
   - `relaySaturationEventsPending`,
   - `relaySaturationEventsTotal`.
4. Snapshot writes are emitted via temporary file + rename to avoid partial reads of in-progress writes.
5. Added relay saturation total counter (`AtomicU64`) in front process:
   - increments on queue saturation, no-healthy-worker path, and connect-failure path,
   - existing pending saturation counter semantics (autoscale-consumed) are preserved.

## Why

Cluster tuning/debug cycles needed a quick, machine-readable operator view of current worker topology and relay pressure without parsing logs or attaching external profilers.

This output gives deterministic local telemetry and reduces manual inspection overhead during autoscale/proxy tuning.

## Validation

1. Focused command tests:
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
   - `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
2. Manual runtime smoke:
   - `timeout 6 target/debug/sec4 run --path examples/lasm-alpha-full --backend lasm --port 18111 --instances 2 --autoscale-max-instances 3 --cluster-status-json <tmp_path>`
   - verified status file presence and JSON payload keys/values (`mode`, `listenPort`, worker and saturation fields).
