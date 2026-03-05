# 1027 M39 Slice: LASM Worker Dispatch via Crossbeam Bounded Channels

This slice removes receiver-lock bottlenecks from LASM request dispatch paths.

## What changed

1. Added `crossbeam-channel` dependency in `sec4-cli`.
2. Replaced std `sync_channel` + `Arc<Mutex<Receiver<TcpStream>>>` fan-out with:
   - `crossbeam_channel::bounded` queues,
   - cloned receivers per worker thread.
3. Applied this change to both:
   - LASM cluster proxy relay worker pool,
   - LASM backend worker pool (`cmd_run_lasm_backend` non-oneshot path).

## Why

The old dispatch path serialized every worker `recv()` call behind a mutex around a single receiver. Under load this adds avoidable coordination overhead and lock contention at the front of the request pipeline.

Crossbeam bounded channels keep the same backpressure model while allowing true multi-consumer blocking receives.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request -- --exact`
- `cargo test -p sec4 --test commands run_command_oneshot_serves_request_and_exits -- --exact`
- local proxy load probe (`wrk`, auth `/health`) remained stable around `~60k req/s`.

The 1M req/s target remains open; this slice removes one known synchronization bottleneck from current dispatch.
