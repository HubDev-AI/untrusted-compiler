# 1018 M39 Slice: LASM Run Cluster Front Proxy and Autoscale Controls

This slice adds built-in horizontal runtime automation to `sec4 run --backend lasm` so multi-instance execution is available from the CLI (not only as external ops guidance).

## What changed

1. Extended `sec4 run` LASM flags:
   - `--instances <N>` (default `1`)
   - `--autoscale-max-instances <N>`
   - `--autoscale-target-connections <N>`
   - `--autoscale-check-ms <N>`
2. Added LASM cluster execution path:
   - parent process runs a TCP front proxy on the requested `--port`,
   - worker LASM server processes are spawned on internal ports,
   - incoming connections are forwarded round-robin to healthy workers.
3. Added adaptive worker scaling:
   - worker count scales between `instances` and `autoscale-max-instances` using live active-connection pressure and target connections per instance.
4. Added deterministic guardrails:
   - cluster mode is rejected with `--oneshot`,
   - autoscale max must be `>= instances`,
   - new cluster/autoscale flags remain LASM-only (`--backend c` rejects them).

## Why

Before this slice, horizontal scaling existed mainly as documented guidance. This change packages a first executable front-layer for LASM so scaling workflows are directly runnable from the CLI.

## Validation

Targeted command coverage added:

- `run_command_rejects_instances_with_c_backend`
- `run_command_rejects_cluster_mode_with_oneshot`
- `run_command_rejects_autoscale_max_instances_less_than_instances`
- `run_command_lasm_cluster_mode_serves_request`

Additional local load checks were run against the new cluster mode; measured throughput is still far below 1M req/s, so hot-path optimization remains an open follow-up.
