# M38-S151 HTTP Max-Concurrency Env/Security Fixture-Helper Adoption

## What it is

M38-S151 completes helper adoption for env fallback and security-header parity fixture build paths.

## Why it exists

To keep all max-concurrency harness branches on one fixture-build surface and reduce drift risk.

## How it works

- `run_http_runtime_health_with_max_concurrency_env(...)` now builds fixtures via `build_c_bin_fixture(...)`.
- Security-header parity fixture path also uses `build_c_bin_fixture(...)`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Next

- Expand helper-layer regression guards (`M38-S152`).
