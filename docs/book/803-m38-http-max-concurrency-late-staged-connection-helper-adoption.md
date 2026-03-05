# M38-S176 HTTP Max-Concurrency Late Staged-Connection Helper Adoption

## What it is

M38-S176 migrates late contention setup to staged first/second connection helpers.

## Why it exists

To eliminate repeated staged connection setup code in late default/low-timeout tests.

## How it works

- Late default and low-timeout tests now call staged first/second connection helpers.
- Late ordered contention contracts are unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_`
