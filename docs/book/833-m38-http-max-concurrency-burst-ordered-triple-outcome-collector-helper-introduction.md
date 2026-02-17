# M38-S206 HTTP Max-Concurrency Burst Ordered Triple Outcome Collector Helper Introduction

## What it is

M38-S206 introduces a collector helper for burst staged triple outcomes.

## Why it exists

Burst staged contention repeated backlog staging, triple response reads, and child wait handling.

## How it works

- Added `collect_burst_ordered_triple_attempt_outcome(...)`.
- Helper stages burst backlog, reads ordered triple responses, and waits for child exit with deterministic timeout envelope.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
