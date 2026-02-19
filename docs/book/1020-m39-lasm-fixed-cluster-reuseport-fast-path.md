# 1020 M39 Slice: LASM Fixed-Cluster Reuse-Port Fast Path

This slice improves LASM horizontal scaling performance by adding a shared-port fixed-cluster mode that avoids front-proxy relay overhead.

## What changed

1. Added internal `sec4 run` LASM `--reuse-port` binding path (hidden/internal flag).
2. Cluster mode behavior split:
   - dynamic cluster (`instances < autoscale-max-instances`) keeps front-proxy + autoscale path,
   - fixed cluster (`instances == autoscale-max-instances`) now launches workers on one shared port via `SO_REUSEPORT` and skips front-proxy relay.
3. Reduced proxy relay overhead in dynamic-cluster mode by simplifying stream clone/copy flow.
4. Added command guard coverage for new LASM-only `--reuse-port` path.

## Why

The front-proxy cluster path was functionally correct but left measurable throughput on the table for fixed multi-instance runs. Shared-port worker binding removes one network hop and avoids proxy dispatch contention in that mode.

## Validation

Targeted command checks:

- `run_command_rejects_reuse_port_with_c_backend`
- existing cluster guard + serve tests remain green.

Local load probe snapshots (`/ping`, wrk, local machine):

- single LASM instance: about `58k req/s`
- proxy-cluster (`instances=4`, autoscale range): about `53k req/s`
- fixed reuse-port cluster (`instances=4`): about `60k req/s`
- fixed reuse-port cluster (`instances=8`, heavier wrk profile): about `96k req/s`

The 1M req/s target is still open; this slice establishes a faster baseline for next tuning steps.
