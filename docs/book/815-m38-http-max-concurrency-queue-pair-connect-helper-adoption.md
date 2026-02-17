# M38-S188 HTTP Max-Concurrency Queue Pair-Connect Helper Adoption

## What it is

M38-S188 migrates queue pair connect/timeout setup to the pair-connect helper.

## Why it exists

To remove duplicated pair connect + timeout setup in queue default and low-timeout branches.

## How it works

- Queue default and low-timeout branches now call `connect_pair_streams_or_terminate(...)`.
- Queue contention contracts remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_`
