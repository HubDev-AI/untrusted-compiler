# M38-S204 HTTP Max-Concurrency Ordered Triple-Attempt Outcome Envelope Introduction

## What it is

M38-S204 adds an ordered triple-attempt outcome envelope used by burst staged contention paths.

## Why it exists

Burst staged contention validates ordered three-response traces and needs one canonical status/response envelope.

## How it works

- Added `OrderedTripleAttemptOutcome` carrying child exit status and ordered first/second/third responses.
- Added `OrderedTripleContract` alias for ordered triple contract signatures.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
