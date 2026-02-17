# M38-S209 HTTP Max-Concurrency Ordered Pair Outcome Observation Formatter Helper Introduction

## What it is

M38-S209 introduces an ordered pair observation formatter helper.

## Why it exists

Late staged pair tests should emit identical observation envelopes without inline formatting duplication.

## How it works

- Added `format_ordered_pair_outcome_observation(...)`.
- Late staged pair loops now update failure observation strings through the helper.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
