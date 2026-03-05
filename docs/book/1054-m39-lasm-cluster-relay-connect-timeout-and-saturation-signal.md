# 1054 M39 Slice: LASM Cluster Relay Connect Timeout and Saturation Signal

This slice hardens LASM cluster relay failure behavior for faster recovery signals under worker churn and connect-path stalls.

## What changed

1. In `compiler/sec4-cli/src/main.rs`, relay threads now increment saturation events when:
   - no worker ports are available in the current snapshot,
   - backend worker connect attempts fail.
2. Backend connect path now uses bounded timeout:
   - `TcpStream::connect_timeout(..., 250ms)`
   instead of potentially unbounded connect calls.
3. Existing unavailable responses are preserved (`503`) and continue using deterministic error envelopes.

## Why

Under failure bursts, unbounded backend connect calls can tie up relay workers, and missing saturation feedback delays autoscale reactions. This slice keeps connect attempts bounded and converts relay failures into scaling pressure signals.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`

## Notes

- This improves responsiveness under failure and saturation conditions; it does not complete the overall 1M req/s tuning target.
