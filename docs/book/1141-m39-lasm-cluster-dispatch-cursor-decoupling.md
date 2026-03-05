# 1141 M39 Slice: LASM Cluster Dispatch Cursor Decoupling

This slice removes shared atomic contention between listener shard dispatch and relay backend selection.

## What changed

1. Updated `dispatch_lasm_cluster_relay_stream` to accept an explicit start index instead of mutating a shared atomic counter.
2. Added listener-local round-robin cursor (`relay_dispatch_round_robin`) advanced per accepted batch.
3. Listener shard dispatch now computes per-connection start index from local batch offset while preserving full fallback scan for saturated/disconnected shards.
4. Relay worker backend-port selection keeps its own atomic counter and is no longer contended by listener enqueue decisions.

## Why

After sharded relay queues, listener dispatch and backend-port selection were still sharing one atomic counter. That introduced avoidable contention and coupled two unrelated scheduling concerns.

Using a listener-local cursor keeps dispatch deterministic and removes the extra cross-thread atomic pressure on the relay hot path.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
3. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-dispatch-decouple.json`
4. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-dispatch-decouple-r2.json`
5. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 20s --target-requests 600000 --skip-build --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-dispatch-decouple-r3.json`
