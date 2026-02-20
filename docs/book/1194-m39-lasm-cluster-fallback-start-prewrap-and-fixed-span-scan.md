# 1194 M39 Slice: LASM Cluster Fallback Start Prewrap and Fixed-Span Scan

This slice tightens the relay fallback dispatch path by removing extra start-index normalization and per-iteration scan guards.

## What changed

1. Multi-relay accept dispatch now computes fallback start index once as:
   - `stream_dispatch_start + 1`, or
   - `0` when `stream_dispatch_start` is the last relay sender.
2. `dispatch_lasm_cluster_relay_stream_fallback` now consumes pre-wrapped start indexes.
3. Fallback scan now uses fixed split spans:
   - first span: `start..start+first_span_len`
   - second span: `0..scan_remaining`

## Why

This removes modulo normalization and per-iteration remaining checks in the fallback scan loop while preserving sender ordering and fallback semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
