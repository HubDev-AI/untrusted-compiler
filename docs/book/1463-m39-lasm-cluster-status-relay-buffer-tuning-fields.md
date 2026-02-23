# M39: LASM cluster status relay buffer tuning fields

## What changed

Relay buffer tuning resolution is now centralized at LASM cluster bootstrap and published in status JSON.

Runtime values now resolved once in cluster startup and threaded into relay workers:

- `SEC4_RT_LASM_CLUSTER_RELAY_BUFFER_BYTES`
- `SEC4_RT_LASM_CLUSTER_RELAY_BUFFER_POOL_MAX`
- `SEC4_RT_LASM_CLUSTER_RELAY_BUFFER_POOL_PREWARM`

Cluster status snapshots now include:

- `relayBufferBytes`
- `relayBufferPoolMax`
- `relayBufferPoolPrewarm`

## Why

Relay worker tuning already existed, but operators had no direct status signal proving which effective values were active at runtime.

Publishing these fields makes capacity runs and production debugging deterministic: status artifacts now show the exact relay buffer memory knobs in use.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
