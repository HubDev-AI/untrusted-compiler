# 218 M10 Slice: Node Baseline Service

This chapter documents the first concrete comparison implementation in M10: a runnable Node baseline service.

## What it is

Added `benchmark-suite/services/node` with:
- `server.mjs` HTTP service implementing benchmark endpoint contract,
- `package.json` start script,
- `smoke.sh` runtime validation script,
- service-specific README.

Implemented endpoints:
- `GET /ping`
- `POST /decode`
- `POST /users`
- `GET /users/:id`

## Why it exists

M10 requires cross-language comparisons. A runnable Node baseline is the first concrete non-Untrusted<T> comparator and unblocks harness integration checks (`bench-profile`, summary generation, report bundling).

## How it works internally

1. Uses Node built-in `http` server (no framework dependency) for low setup friction.
2. Implements spec-aligned validation for `/decode` payload fields.
3. Uses in-memory map for `/users` create/read contract behavior.
4. Emits simple standard error envelope shape on validation/not-found/internal paths.
5. `smoke.sh` launches service on isolated port and verifies:
   - `/ping` returns `ok`,
   - `/decode` accepts canonical payload and returns `ok=true`.

## Inputs, outputs, and constraints

- Inputs:
  - payload spec from `benchmark-suite/spec/payloads/user_4kb.json`.
- Outputs:
  - runnable baseline service process on `PORT` (default `8080`).
- Constraints:
  - `/users` storage is in-memory in this slice.
  - DB-backed parity is deferred to follow-up M10 work.

## Failure modes and diagnostics

- invalid JSON -> `400 JSON.INVALID_SYNTAX`
- payload validation mismatch -> `400 VALIDATION.INVALID`
- unknown route -> `404 HTTP.NOT_FOUND`
- uncaught runtime errors -> `500 HTTP.INTERNAL`

## Example usage

```bash
cd benchmark-suite/services/node
npm run start
```

Smoke check:

```bash
benchmark-suite/services/node/smoke.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - in-memory `/users` path keeps setup simple but is not DB-parity benchmark behavior.
- Next:
  - wire `/users` endpoints to benchmark Postgres schema for true read/write workload parity,
  - add equivalent runnable baselines for Go and Rust services.
