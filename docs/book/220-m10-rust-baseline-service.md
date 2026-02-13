# 220 M10 Slice: Rust Baseline Service

This chapter documents the Rust comparison implementation for the M10 benchmark harness.

## What it is

Added `benchmark-suite/services/rust` with:
- `Cargo.toml`
- `src/main.rs`
- `smoke.sh`
- updated service README.

Implemented endpoints:
- `GET /ping`
- `POST /decode`
- `POST /users`
- `GET /users/:id`

## Why it exists

M10 requires cross-language baselines with comparable behavior. Rust is a key performance/safety reference, so a runnable Rust contract service is required for comparative runs.

## How it works internally

1. Uses `tiny_http` for a minimal synchronous HTTP server.
2. Uses `serde_json` for JSON decode/encode.
3. Uses in-memory `Mutex<HashMap<...>>` for `/users` write/read contract.
4. Validates payload fields according to benchmark spec constraints (uuid v4, email, age, tags, zip).
5. Emits standard envelope-style errors with `code/kind/message/status/traceId/timeMs`.
6. `smoke.sh` does readiness polling and validates:
   - `/ping` returns `ok`,
   - `/decode` succeeds with canonical payload.

## Inputs, outputs, and constraints

- Inputs:
  - `benchmark-suite/spec/payloads/user_4kb.json`
- Outputs:
  - runnable Rust baseline service on `PORT` (default `8080`).
- Constraints:
  - current `/users` backend is in-memory (not Postgres-backed yet).
  - crate is self-contained via local `[workspace]` to avoid parent workspace coupling.

## Failure modes and diagnostics

- malformed JSON -> `400 JSON.INVALID_SYNTAX`
- payload validation mismatch -> `400 VALIDATION.INVALID`
- invalid user id path -> `400 VALIDATION.UUID_INVALID`
- missing user -> `404 HTTP.NOT_FOUND`
- unknown route -> `404 HTTP.NOT_FOUND`

## Example usage

```bash
cd benchmark-suite/services/rust
cargo run
```

Smoke test:

```bash
benchmark-suite/services/rust/smoke.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - synchronous tiny_http keeps setup simple but is not representative of a tuned production Rust stack yet.
- Next:
  - add Postgres-backed `/users` path for DB parity,
  - optionally add an `axum` implementation variant for ecosystem-realistic comparison.
