# M38-S149 HTTP Max-Concurrency Queue Fixture-Helper Adoption

## What it is

M38-S149 migrates queue-boundary fixture setup (default and low-timeout variants) to shared fixture helper.

## Why it exists

Queue tests still carried local fixture scaffolding after helper introduction.

## How it works

- Queue fixture setup now uses `build_c_bin_fixture(...)` in both queue variants.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

- Apply fixture helper on late/burst paths (`M38-S150`).
