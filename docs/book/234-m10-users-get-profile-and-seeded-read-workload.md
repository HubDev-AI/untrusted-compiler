# 234 M10 Slice: Users-Get Profile and Seeded Read Workload

This chapter documents adding `users-get` as a first-class benchmark profile in M10, including deterministic pre-seeding of the read target user.

## What it is

Updated:
- `benchmark-suite/scripts/run_profile.sh`
- `benchmark-suite/scripts/run_comparison_matrix.sh`
- `benchmark-suite/load/wrk2/get_user.lua`
- `benchmark-suite/load/wrk2/post_decode.lua`
- `benchmark-suite/load/wrk2/post_users.lua`
- `benchmark-suite/scripts/test_run_profile.sh`
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- added profile endpoint `users-get` to `run_profile.sh`,
- added deterministic seed step for `users-get` (`POST /users` before load),
- added `BENCH_USER_ID` support in `get_user.lua`,
- added `BENCH_PAYLOAD_FILE` support in POST Lua scripts to avoid cwd-coupled payload resolution,
- wired orchestrator to run `users-get` for every implementation,
- added `BENCH_TARGET_USERS_GET` override and Makefile target `bench-users-get`.

## Why it exists

The M10 benchmark spec requires both DB write and DB read workloads (`POST /users` and `GET /users/:id`). The harness previously executed only write-path load tests. This slice closes that gap and makes read-path comparisons part of default matrix execution.

## How it works internally

1. `run_profile.sh users-get` builds a `wrk2` command with `load/wrk2/get_user.lua`.
2. Before starting load, it seeds one deterministic user via `POST /users` using `spec/payloads/user_4kb.json`.
3. Seeded user id is extracted from payload JSON and injected as `BENCH_USER_ID` for the Lua request script.
4. Orchestrator now runs:
   - `ping`
   - `decode`
   - `users-post`
   - `users-get`
5. Summaries and report bundles include `users-get` like any other endpoint.

## Inputs, outputs, and constraints

- Inputs:
  - profile endpoint `users-get`,
  - seed payload `benchmark-suite/spec/payloads/user_4kb.json`,
  - optional target override `BENCH_TARGET_USERS_GET`.
- Outputs:
  - `results/raw/<impl>-users-get.txt`
  - `results/summaries/<impl>-users-get.json`
- Constraints:
  - `users-get` non-dry-run requires `curl` and `jq` (for seed setup),
  - benchmark service must implement `POST /users` and `GET /users/:id` contract.

## Failure modes and diagnostics

- missing payload file -> explicit profile error.
- missing payload id field -> explicit profile error.
- seed request non-success (`200/201/409` expected) -> explicit profile error.
- missing `curl`/`jq` in non-dry-run `users-get` -> explicit dependency error.

## Example usage

```bash
make -C benchmark-suite bench-profile IMPL=ailang ENDPOINT=users-get
```

With override:

```bash
BENCH_TARGET_USERS_GET=5000 make -C benchmark-suite bench-profile IMPL=node ENDPOINT=users-get
```

## Tradeoffs and next steps

- Tradeoff:
  - profile runner now performs a seed request before read-load execution, adding a small setup dependency on endpoint contract availability.
- Next:
  - add optional multi-id read distribution profile for cache-sensitive comparisons,
  - add explicit read/write mix benchmark profile once baseline single-endpoint matrix stabilizes.
