# 406 M16 Slice: res.okMeta Runtime Envelope Coverage

This chapter documents M16-S11: adding explicit live-runtime coverage for `res.okMeta(...)`.

## What it is

A new runtime integration test in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_res_ok_meta_includes_meta_when_clang_available`

The test validates the `res.okMeta(...)` path over a real HTTP roundtrip in oneshot runtime mode.

## Why it exists

`res.okMeta(...)` had compile/lowering coverage and shared runtime helper logic, but it lacked dedicated end-to-end HTTP validation. This slice closes that gap and ensures the `meta` branch of the success envelope is explicitly protected against regressions.

## Implementation details

1. Added temp-project fixture route:
   - `req.json("CreateUserRequest")`
   - `res.okMeta(201, "CreateUserResponse", 1, 2)`
2. Test sends `POST /users` and asserts:
   - `HTTP/1.1 201 Created`
   - JSON content-type
   - success envelope fields:
     - `ok`
     - `status`
     - `traceId`
     - `data`
     - `meta`

## Validation

The new test is green and confirms deterministic envelope+meta behavior on the live runtime path.

## Tradeoffs and next steps

- This slice improves runtime confidence without changing production logic.
- Future slices can extend `meta` coverage to richer typed serialization once runtime value materialization moves beyond placeholders.
