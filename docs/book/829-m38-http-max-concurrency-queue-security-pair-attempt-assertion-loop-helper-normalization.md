# M38-S202 HTTP Max-Concurrency Queue/Security Pair-Attempt Assertion-Loop Helper Normalization

## What it is

M38-S202 introduces a shared pair-attempt assertion-loop helper for queue/security max-concurrency contention tests.

## Why it exists

To remove duplicated per-attempt loop plumbing across queue/security branches and keep contention checks aligned.

## How it works

- Added `run_pair_contention_attempt_loop(...)` as the canonical pair-attempt loop.
- Queue default, queue low-timeout, and security parity tests now call the shared loop helper.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
