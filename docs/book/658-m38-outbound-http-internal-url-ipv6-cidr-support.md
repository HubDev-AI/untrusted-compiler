# M38-S38 Outbound HTTP Internal-URL IPv6 CIDR Support

## What it is

M38-S38 extends internal allowlist CIDR support from IPv4-only to dual-stack (IPv4 + IPv6) and enables bracketed IPv6 host parsing for internal URL policy checks.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Internal allowlist CIDR checks previously accepted only IPv4 CIDRs and could not evaluate IPv6 literal hosts. That limited policy precision for dual-stack/internal deployments.

## How it works

1. Added IPv6 CIDR parsing and prefix matching helpers.
2. Upgraded internal CIDR list validation to accept both IPv4 and IPv6 CIDR tokens.
3. Upgraded CIDR matching path to match host literals against both IPv4 and IPv6 CIDR entries.
4. Extended URL host parsing/port resolution support for bracketed IPv6 literal hosts used by `url.internal(...)` checks.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_url_internal_respects_allowed_cidrs_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_url_public_respects_env_policy_lists_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- CIDR validator/matcher logic is more complex due dual-stack handling.
- This slice focuses on policy gates (`url.internal`), not full outbound HTTP GET IPv6 transport path.

## Next

1. Extend outbound HTTP GET parser/connection path to fully support bracketed IPv6 literal URLs.
2. Add deterministic diagnostics for malformed IPv6-literal request URLs in runtime net request flow.
