# M38-S120 HTTP Ingress Override Input Hardening

## What it is

M38-S120 adds deterministic fail-fast validation for invalid zero-value HTTP ingress override flags in `sec4 run`.

Files:

- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

`sec4 run` already supports HTTP ingress override flags (`--max-body-bytes`, `--serve-timeout-ms`), but zero values were accepted and delegated to runtime behavior, which made invalid-override behavior ambiguous and delayed.

This slice hardens the CLI boundary so invalid override inputs fail immediately with deterministic diagnostics.

## How it works

1. `cmd_run` now validates override values before any project validation/build/runtime launch:
   - `--max-body-bytes` must be `>= 1`
   - `--serve-timeout-ms` must be `>= 1`
2. Invalid inputs produce stable `run failed: ... must be >= 1` diagnostics and exit with deterministic non-zero status.
3. Added command integration tests to lock both invalid-zero branches.

## Validation

- `cargo test -p sec4 --test commands run_command_rejects_zero_max_body_bytes_override`
- `cargo test -p sec4 --test commands run_command_rejects_zero_serve_timeout_ms_override`
- `cargo test -p sec4 --test commands`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Introduces stricter CLI input validation, which can break previously tolerated invalid invocation patterns.
- Improves determinism and operator feedback by preventing ambiguous runtime-side fallback behavior.

## Next

1. Bridge `http.max_header_bytes` into runtime materialization and enforce deterministic header-size limits.
2. Add run-command/runtime coverage for policy-driven header-size rejection behavior.
