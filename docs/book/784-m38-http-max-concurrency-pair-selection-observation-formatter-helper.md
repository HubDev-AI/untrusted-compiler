# M38-S157 HTTP Max-Concurrency Pair-Selection and Observation Formatter Helper

## What it is

M38-S157 adds canonical pair-selection and attempt-observation formatter helpers.

## Why it exists

To keep contention failure envelopes readable and deterministic while avoiding duplicated branching logic.

## How it works

- `select_success_and_throttle(...)` resolves success/throttle pair assignment deterministically.
- `format_two_response_observation(...)` and `format_three_response_observation(...)` normalize failure output envelopes.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
