# M38-S150 HTTP Max-Concurrency Late/Burst Fixture-Helper Adoption

## What it is

M38-S150 migrates late-connection and burst-ingress fixture setup (default and low-timeout variants) to shared fixture helper.

## Why it exists

Late/burst tests still carried repeated fixture scaffolding.

## How it works

- Late and burst fixture setup now flows through `build_c_bin_fixture(...)`.
- Runtime contract assertions remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

- Complete helper adoption for env/security fixture paths (`M38-S151`).
