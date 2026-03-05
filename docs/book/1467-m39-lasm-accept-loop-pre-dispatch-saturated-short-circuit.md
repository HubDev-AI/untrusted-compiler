# M39: LASM accept-loop pre-dispatch saturated short-circuit

## What changed

LASM cluster accept loop now adds a pre-dispatch saturated fast path in multi-relay mode.

Behavior:

- when saturation flags are already active for the batch/recent window,
- and the preferred relay sender is live,
- and that sender queue is already full (`Sender::is_full()`),

accept loop now short-circuits directly to deterministic saturated handling (`503`) before invoking primary `try_send` + fallback dispatch routines.

## Why

Under sustained queue pressure, many requests were still paying dispatch-attempt overhead even though the preferred sender state already implied saturated outcome.

This change trims avoidable hot-path dispatch work while preserving current saturation semantics and counters.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
