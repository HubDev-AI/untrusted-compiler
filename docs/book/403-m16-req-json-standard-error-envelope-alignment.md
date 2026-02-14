# 403 M16 Slice: req.json Standard Error Envelope Alignment

This chapter documents M16-S8: migrating runtime `req.json(...)` gate failures from ad-hoc error payloads to the standard structured error envelope.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` introducing a helper that emits deterministic standard error JSON responses for gate failures:

- `error.code`
- `error.kind`
- `error.message`
- `error.status`
- `error.traceId`
- `error.timeMs`

## Why it exists

`req.json(...)` runtime gate failures previously returned plain one-field payloads (`{"error":"..."}`). That shape was not aligned with the project’s standard runtime error model and reduced consistency for logs, tooling, and debugging workflows.

## Implementation details

1. Added helper:
   - `sec4_rt_store_std_error_response(status, code, kind, message)`.
2. Migrated `req.json(...)` failure branches to structured errors:
   - missing body: `JSON.BODY_REQUIRED` (`400`, `validation`)
   - invalid body: `JSON.INVALID_BODY` (`400`, `validation`)
   - invalid content-type: `HTTP.CONTENT_TYPE_INVALID` (`415`, `validation`)
   - oversized body: `LIMIT.BODY_BYTES` (`413`, `resource_limit`)
3. Kept payload deterministic:
   - fixed `traceId` (`rt_trace`) and `timeMs` (`0`) in this runtime bootstrap stage.

## Validation

Updated runtime integration assertions in `compiler/sec4-cli/tests/json_output.rs` to check stable code/message pairs for:

- invalid JSON body path,
- invalid content-type path,
- oversized body path.

All targeted runtime gate tests pass with the new envelope.

## Tradeoffs and next steps

- This slice improves contract consistency without adding dynamic trace/time providers yet.
- Future slices can replace deterministic placeholders with real request `traceId` propagation and runtime clock values while keeping envelope shape stable.
