# M39: LASM cluster relay buffer size tuning

## What changed

LASM cluster relay pump buffers are now runtime-tunable instead of fixed-size.

New runtime env control:

- `SEC4_RT_LASM_CLUSTER_RELAY_BUFFER_BYTES`

Resolved value is clamped to `1024..1048576` bytes and threaded through:

- relay connection initialization,
- pooled relay buffer reuse (`new_with_buffers`),
- fresh relay buffer allocations (`new`).

## Why

Relay pump throughput and memory footprint trade off directly against per-direction buffer size.
A fixed compile-time value prevented operator tuning for different workloads and machine sizes.

With this change, operators can tune hot-path proxy behavior without code changes.

## Behavioral notes

- Existing relay logic and deterministic dispatch behavior are unchanged.
- Buffer pools now normalize pooled buffer sizes to the resolved runtime budget before reuse.
- Default runtime behavior uses the relay-pump default buffer budget and remains backward-compatible.
