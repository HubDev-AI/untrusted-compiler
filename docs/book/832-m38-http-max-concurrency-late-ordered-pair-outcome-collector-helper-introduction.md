# M38-S205 HTTP Max-Concurrency Late Ordered Pair Outcome Collector Helper Introduction

## What it is

M38-S205 introduces a collector helper for late staged pair outcomes.

## Why it exists

Late staged contention repeated backlog staging, response reads, and child wait handling across tests.

## How it works

- Added `collect_late_ordered_pair_attempt_outcome(...)`.
- Helper stages late backlog, reads ordered pair responses, and waits for child exit with deterministic timeout envelope.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
