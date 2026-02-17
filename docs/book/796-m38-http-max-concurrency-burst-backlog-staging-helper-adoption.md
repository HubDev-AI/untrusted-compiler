# M38-S169 HTTP Max-Concurrency Burst-Backlog Staging Helper Adoption

## What it is

M38-S169 centralizes burst-ingress backlog staging for the second/third connections.

## Why it exists

To keep burst default and low-timeout request staging deterministic and synchronized.

## How it works

- `stage_burst_backlog_requests(...)` stages second/third backlog requests and noise, then releases first request.
- Both burst contention tests now use this helper.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_`
