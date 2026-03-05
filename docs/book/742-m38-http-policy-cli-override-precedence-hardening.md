# M38-S119 HTTP Policy-vs-CLI Override Precedence Hardening

## What it is

M38-S119 locks the precedence contract between policy-provided HTTP ingress values and `sec4 run` CLI override flags.

Files:

- `compiler/sec4-cli/tests/commands.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

After M38-S118, HTTP ingress values are policy-driven by default (`http.max_body_bytes`, `http.default_timeout_ms`). The next correctness risk is precedence drift: CLI override flags could silently stop overriding policy values, or policy values could stop applying when flags are absent.

This slice codifies both branches as integration contracts.

## How it works

1. Added run-command e2e for body-limit override precedence:
   - policy sets `http.max_body_bytes = 32`
   - CLI passes `--max-body-bytes 4096`
   - request payload larger than 32 bytes is accepted (`201`) to prove CLI override wins.
2. Added run-command e2e for timeout override precedence:
   - policy sets `http.default_timeout_ms = 20000`
   - CLI passes `--serve-timeout-ms 1`
   - oneshot run exits quickly and deterministically, proving CLI timeout override wins over policy timeout.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_cli_max_body_bytes_overrides_policy_limit`
- `cargo test -p sec4 --test commands run_command_oneshot_cli_serve_timeout_overrides_policy_timeout`
- `cargo test -p sec4 --test commands`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds more integration-test runtime cost in `commands` suite.
- Significantly reduces regression risk for real operator behavior, because precedence guarantees are now executable contracts.

## Next

1. Add deterministic CLI input hardening for invalid zero override values.
2. Ensure non-zero override validation fails fast with stable diagnostics before runtime launch.
