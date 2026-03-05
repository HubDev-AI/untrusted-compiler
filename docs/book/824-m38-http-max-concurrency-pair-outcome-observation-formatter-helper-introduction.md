# M38-S197 HTTP Max-Concurrency Pair Outcome Observation Formatter Helper Introduction

## What it is

M38-S197 introduces a helper for rendering pair outcome observations.

## Why it exists

To centralize queue/security attempt observation formatting after outcome helper adoption.

## How it works

- Added `format_pair_outcome_observation(...)`.
- Helper wraps canonical two-response attempt observation formatter.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
