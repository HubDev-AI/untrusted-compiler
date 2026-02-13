# 219 M10 Slice: Go Baseline Service

This chapter documents the next runnable comparator in M10: a Go baseline service.

## What it is

Added `benchmark-suite/services/go` with:
- `main.go` HTTP server,
- `go.mod`,
- `smoke.sh`,
- updated README.

Implemented endpoints:
- `GET /ping`
- `POST /decode`
- `POST /users`
- `GET /users/:id`

## Why it exists

M10 comparison credibility improves with multiple runnable baselines. Adding Go establishes the expected “simple + fast backend” reference in the benchmark harness.

## How it works internally

1. Uses standard library `net/http` server (no framework dependency).
2. Implements contract-aligned JSON validation for decode/users payloads.
3. Stores users in a mutex-guarded in-memory map for read/write endpoint parity.
4. Emits standard error envelope fields (`code`, `kind`, `message`, `status`, `traceId`, `timeMs`).
5. `smoke.sh` performs readiness polling and validates:
   - `/ping` returns `ok`,
   - `/decode` succeeds with canonical payload.

## Inputs, outputs, and constraints

- Inputs:
  - `benchmark-suite/spec/payloads/user_4kb.json`.
- Outputs:
  - runnable Go baseline service (default port `8080`, configurable with `PORT`).
- Constraints:
  - current `/users` storage is in-memory in this slice.
  - benchmark DB schema parity for `/users` is deferred.

## Failure modes and diagnostics

- malformed JSON -> `400 JSON.INVALID_SYNTAX`
- validation mismatch -> `400 VALIDATION.INVALID`
- invalid user id path -> `400 VALIDATION.UUID_INVALID`
- unknown route -> `404 HTTP.NOT_FOUND`
- runtime failures -> `500` envelope path

## Example usage

```bash
cd benchmark-suite/services/go
go run .
```

Smoke test:

```bash
benchmark-suite/services/go/smoke.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - in-memory persistence keeps service setup minimal but is not DB-comparable for final write/read benchmarks.
- Next:
  - wire Postgres path for `/users` endpoints,
  - add Rust baseline service implementation for another low-level comparator.
