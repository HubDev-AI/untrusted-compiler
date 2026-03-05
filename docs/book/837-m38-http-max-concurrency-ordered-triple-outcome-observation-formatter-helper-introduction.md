# M38-S210 HTTP Max-Concurrency Ordered Triple Outcome Observation Formatter Helper Introduction

## What it is

M38-S210 introduces an ordered triple observation formatter helper.

## Why it exists

Burst staged triple tests should emit identical observation envelopes without inline formatting duplication.

## How it works

- Added `format_ordered_triple_outcome_observation(...)`.
- Burst staged triple loops now update failure observation strings through the helper.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
