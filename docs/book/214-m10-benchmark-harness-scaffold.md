# 214 M10 Slice: Benchmark Harness Scaffold

This chapter documents the first M10 implementation slice: a reproducible benchmark harness scaffold.

## What it is

Added `benchmark-suite/` with initial reproducibility infrastructure:
- endpoint contract and payload spec,
- Postgres schema,
- wrk2 load scripts,
- shared docker-compose DB,
- orchestration Makefile,
- result directory structure and environment-capture script,
- placeholder service directories for AILang/Go/Node/Rust/C.

## Why it exists

M10 starts with harness reproducibility before cross-language implementation details. This scaffold creates a single contract for workload parity and reduces benchmark-theater risk.

## How it works internally

1. `spec/` defines behavior and payloads:
   - `spec/endpoints.md`
   - `spec/payloads/*.json`
   - `spec/db/schema.sql`
2. `load/wrk2/` provides scripts for request profiles:
   - `post_decode.lua`
   - `post_users.lua`
   - `get_user.lua`
3. `Makefile` defines baseline targets:
   - `db-up`, `db-down`, `db-schema`, `env`,
   - `bench-ping`, `bench-decode`, `bench-users`.
4. `scripts/capture_env.sh` writes standardized environment metadata.
5. `results/` stores raw/summaries/plots outputs; root ignore rule prevents accidental host-specific env output commits.

## Inputs, outputs, and constraints

- Inputs:
  - benchmark endpoint and payload spec from chapter 71.
- Outputs:
  - `benchmark-suite/` scaffold ready for per-language service implementations.
- Constraints:
  - scaffold does not yet include full service implementations for all languages.
  - benchmark commands expect target services to be running at `http://127.0.0.1:8080`.

## Failure modes and diagnostics

- Missing external tools (`wrk2`, `psql`, `docker`) fail corresponding targets directly with tool-native errors.
- Missing service implementations produce connection failures during benchmark targets (expected at scaffold stage).

## Example usage

```bash
make -C benchmark-suite help
make -C benchmark-suite db-up
make -C benchmark-suite db-schema
make -C benchmark-suite env
```

Once service implementations are added:

```bash
make -C benchmark-suite bench-ping IMPL=ailang
```

## Tradeoffs and next steps

- Tradeoff:
  - scaffold prioritizes reproducibility contracts first, leaving implementation-specific service wiring for subsequent M10 slices.
- Next:
  - add AILang benchmark service implementation under `benchmark-suite/services/ailang`,
  - add normalized result summary generator (`summary.json`) and first comparative run artifacts.
