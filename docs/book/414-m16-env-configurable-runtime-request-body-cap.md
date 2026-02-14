# 414 M16 Slice: Env-Configurable Runtime Request Body Cap

This chapter documents M16-S19: making runtime request-body cap configurable via environment.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` that reads:

- `SEC4_RT_HTTP_MAX_BODY_BYTES`

and uses it to tighten the effective request-body cap for `req.json(...)` processing when configured value is lower than runtime buffer ceiling.

## Why it exists

Body-size protection existed, but was fixed at compile-time buffer limits. This slice enables deployment-time tightening of limits without changing code, which is useful for environment-specific hardening.

## Implementation details

1. In request capture path, runtime now computes body cap as:
   - min(default runtime body buffer cap, env-configured cap if valid and lower).
2. Existing overflow behavior is preserved:
   - overflow sets `body_limit_exceeded`,
   - `req.json(...)` returns deterministic `413` with `LIMIT.BODY_BYTES`.

## Validation

Added integration test in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_req_json_honors_env_body_limit_when_set`

Flow:
1. Start fixture with `SEC4_RT_HTTP_MAX_BODY_BYTES=16`.
2. Send larger JSON body.
3. Assert deterministic `413 Payload Too Large` and `LIMIT.BODY_BYTES` error payload.

## Tradeoffs and next steps

- This slice adds deployment flexibility while keeping deterministic test behavior.
- Future improvements can connect this to policy-derived runtime budget values for unified config governance.
