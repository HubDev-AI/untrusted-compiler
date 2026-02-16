# M38-S123 HTTP Ingress Generic Body-Limit Enforcement

## What it is

M38-S123 adds deterministic runtime body-limit enforcement for non-JSON request paths that do not call `req.json(...)`.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/commands.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

Previously, body-limit rejection was strongly tied to `req.json(...)` paths. A handler that accepted arbitrary body content and returned a success response without `req.json(...)` could still emit `201` even when body size exceeded configured limits.

This slice closes that gap for generic ingress paths.

## How it works

1. Runtime now checks generic ingress overflow after handler execution:
   - condition: `body_limit_exceeded && !json_checked`
2. When triggered, runtime emits deterministic rejection response:
   - status: `413 Payload Too Large`
   - body: `request body exceeds runtime limit`
3. JSON-specific behavior remains intact:
   - `req.json(...)` continues to emit its own deterministic error envelope (`LIMIT.BODY_BYTES`) when applicable.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_enforces_body_limit_for_non_json_handler_paths`
- `cargo test -p sec4 --test commands run_command_oneshot_applies_http_body_limit_from_policy`
- `cargo test -p sec4 --test commands`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Generic enforcement is currently applied after handler invocation, which preserves `req.json` contracts but does not prevent handler-side work before rejection.
- Still improves correctness at response boundary by blocking oversized-body success responses on non-JSON paths.

## Next

1. Bridge and enforce `http.max_concurrency` runtime behavior.
2. Add deterministic throttling/over-cap acceptance tests for ingress concurrency limits.
