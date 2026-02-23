# M39: LASM cluster relay buffer-pool tuning and prewarm

## What changed

LASM cluster relay workers now have runtime controls for relay buffer-pool sizing and optional startup prewarm.

New runtime env controls:

- `SEC4_RT_LASM_CLUSTER_RELAY_BUFFER_POOL_MAX`
- `SEC4_RT_LASM_CLUSTER_RELAY_BUFFER_POOL_PREWARM`

Behavior:

- `pool_max` default is derived from relay accept-batch sizing (`accept_batch * 4`, minimum `64`) and clamped to `16..65536`.
- `prewarm` default is `0` and clamped to `0..pool_max`.
- Relay worker loop now resolves these values at startup.
- Relay buffer pool is initialized with `pool_max` capacity and optionally prefilled with `prewarm` reusable `(client_to_upstream, upstream_to_client)` buffers at resolved relay buffer size.

## Why

Relay workers already reused buffers after connection completion, but first-burst traffic still paid fresh allocation cost for each new relay pump.

This change gives operators deterministic runtime knobs to trade startup memory for reduced allocation churn under bursty traffic, without changing relay semantics.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
