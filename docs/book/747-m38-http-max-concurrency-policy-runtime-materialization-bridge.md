# M38-S124 HTTP Max-Concurrency Policy/Runtime Materialization Bridge

## What it is

M38-S124 wires `http.max_concurrency` end-to-end from policy parsing through `sec4 run` env bridging into runtime ingress throttling.

Files:

- `compiler/sec4-core/src/policy.rs`
- `compiler/sec4-core/tests/policy.rs`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`
- `runtime/c/sec4_runtime.c`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

`http.max_concurrency` already existed in policy inputs and profile files, but it was not persisted in the typed policy model and never reached runtime behavior. That created a silent config gap: operators could set the value without any ingress effect.

This slice closes that gap and adds deterministic overload signaling.

## How it works

1. Policy persistence and validation:
   - `HttpPolicyConfig` now includes `max_concurrency`.
   - Default baseline is `256`.
   - Parser validates `http.max_concurrency >= 1`.

2. Runtime bridge in `sec4 run`:
   - `cmd_run` exports policy value to runtime env:
     - `SEC4_RT_HTTP_MAX_CONCURRENCY`

3. Runtime ingress enforcement:
   - Runtime parses `SEC4_RT_HTTP_MAX_CONCURRENCY` with bounded fallback.
   - HTTP serve loop keeps a pending-connection queue capped by configured concurrency.
   - Accepted connections over the cap receive deterministic throttle response:
     - status `503 Service Unavailable`
     - body `server busy: max concurrency exceeded`

4. Deterministic command e2e coverage:
   - New test establishes parallel socket connections against oneshot runtime with `max_concurrency = 1`.
   - Assertions require one successful request (`200`) and one deterministic throttle (`503`).

## Validation

- `cargo test -p sec4-core --test policy`
- `cargo test -p sec4 --test commands run_command_oneshot_applies_http_max_concurrency_from_policy`
- `cargo test -p sec4 --test commands`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Ingress handling remains single-threaded; concurrency cap applies to accepted pending-connection queue depth, not parallel request execution threads.
- Over-cap behavior is deterministic and policy-driven, but this is still conservative throttling suitable for alpha runtime constraints rather than high-throughput production tuning.

## Next

1. Add `sec4 run` CLI override for max-concurrency.
2. Pin deterministic policy-vs-CLI precedence tests for max-concurrency.
