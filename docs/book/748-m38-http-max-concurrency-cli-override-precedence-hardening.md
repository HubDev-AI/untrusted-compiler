# M38-S125 HTTP Max-Concurrency CLI Override Precedence Hardening

## What it is

M38-S125 adds an explicit `sec4 run` max-concurrency override and pins deterministic precedence over policy defaults.

Files:

- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

After M38-S124, `http.max_concurrency` was enforced from policy, but operators had no quick runtime override in `sec4 run` for local diagnosis and controlled stress scenarios.

This slice adds that override while preserving deterministic validation and precedence behavior.

## How it works

1. CLI surface:
   - `sec4 run` now accepts:
     - `--max-concurrency <n>`

2. Validation:
   - `--max-concurrency 0` is rejected deterministically:
     - stderr: `run failed: --max-concurrency must be >= 1`
     - exit code: `2`

3. Runtime bridge precedence:
   - Policy value still materializes first:
     - `SEC4_RT_HTTP_MAX_CONCURRENCY=<policy.http.max_concurrency>`
   - CLI override is applied afterward when present:
     - `SEC4_RT_HTTP_MAX_CONCURRENCY=<flag value>`
   - Result: CLI value deterministically overrides policy for runtime ingress throttling.

4. Coverage:
   - Added e2e command test for precedence (`policy=1`, CLI override `2`) in persistent serve mode that asserts two successful responses without `503` throttles.
   - Added deterministic invalid-input test for zero override.

## Validation

- `cargo test -p sec4 --test commands run_command_cli_max_concurrency_overrides_policy_limit`
- `cargo test -p sec4 --test commands run_command_rejects_zero_max_concurrency_override`
- `cargo test -p sec4 --test commands`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Override is runtime-only (`sec4 run`) and intentionally does not mutate policy files.
- Concurrency behavior still inherits oneshot-mode serving constraints; this slice focuses on precedence and operator control, not ingest-loop redesign.

## Next

1. Harden runtime fallback/clamp behavior for malformed/oversized `SEC4_RT_HTTP_MAX_CONCURRENCY` values.
2. Add explicit tests for invalid-env fallback parity.
