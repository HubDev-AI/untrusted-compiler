# 227 M10 Slice: C Baseline Service

This chapter documents the optional C floor comparator implementation for M10.

## What it is

Added:
- `benchmark-suite/services/c/server.c`
- `benchmark-suite/services/c/smoke.sh`
- `benchmark-suite/services/c/README.md`

Updated:
- `benchmark-suite/scripts/run_comparison_matrix.sh` (supports `--impls c`)
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/README.md`

The C service implements the shared contract endpoints:
- `GET /ping`
- `POST /decode`
- `POST /users`
- `GET /users/:id`

## Why it exists

M10 includes an optional C floor reference for benchmarking context. This slice adds a runnable, low-dependency C server so AILang comparisons can include a near-backend-floor baseline.

## How it works internally

1. Runs a minimal socket-based HTTP server (single process, connection-per-request, `Connection: close`).
2. Parses request line, headers, and body (bounded buffers).
3. Validates user payload fields with deterministic checks:
   - UUID v4 format,
   - email shape,
   - age range,
   - tags count/item length,
   - zip digits length,
   - boolean flags `a/b/c` presence.
4. Stores users in an in-memory fixed-size map and returns stored JSON body for `GET /users/:id`.
5. Emits standard JSON error envelope with `traceId` and `timeMs`.

## Inputs, outputs, and constraints

- Inputs:
  - HTTP requests matching endpoint contract.
- Outputs:
  - text/json responses matching benchmark suite expectations.
- Constraints:
  - fixed in-memory user capacity,
  - simplified JSON extraction logic (contract-oriented, not full JSON parser),
  - single-threaded server model.

## Failure modes and diagnostics

- invalid HTTP parse -> `HTTP.BAD_REQUEST`.
- invalid payload fields -> `VALIDATION.INVALID`.
- invalid user id path -> `VALIDATION.UUID_INVALID`.
- missing user -> `HTTP.NOT_FOUND`.
- user store capacity exceeded -> `HTTP.INTERNAL`.

## Example usage

```bash
benchmark-suite/services/c/smoke.sh
```

Manual run:

```bash
cd benchmark-suite/services/c
cc -O2 -std=c11 server.c -o c-bench-server
./c-bench-server
```

Orchestrator include:

```bash
benchmark-suite/scripts/run_comparison_matrix.sh --dry-run --impls node,go,rust,c
```

## Tradeoffs and next steps

- Tradeoff:
  - parser and HTTP handling are intentionally minimal and not production-grade.
- Next:
  - add optional keep-alive support for improved comparability at high RPS,
  - add DB-backed variant if C floor comparison should include persistent-path parity.
