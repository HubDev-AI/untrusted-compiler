# M38-S168 HTTP Max-Concurrency Late-Backlog Staging Helper Adoption

## What it is

M38-S168 centralizes late-connection backlog staging for request and trailing-noise writes.

## Why it exists

To keep late contention setup deterministic across default and low-timeout branches.

## How it works

- `stage_late_backlog_requests(...)` writes backlog request/noise before releasing the accepted request.
- Both late tests reuse the same staging helper.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_`
