# 191 M7 Slice: security_map crypto Namespace Alias Resolution

This chapter documents the M7 alias-resolution hardening for `crypto.*` call forwarding.

## What it is

Extended `security_map` callable alias resolution so `crypto` is treated as a tagged call namespace, just like `db`, `req`, `res`, and other stdlib namespaces.

## Why it exists

Before this slice, forwarded namespace aliases such as:

```ut
fn cryptoNs() { crypto }
let c = cryptoNs();
c.ctEq(a, b);
```

were not normalized to `crypto.ctEq`, so gate tags and argument roles were dropped from metadata.

## How it works internally

1. Added `crypto` to `is_tagged_call_namespace(...)`.
2. Existing alias/forward normalization now accepts `crypto` namespace values as valid callable roots.
3. Existing `call_tags_for(...)` and `call_arg_roles(...)` mappings for `crypto.ctEq` apply automatically after normalization.
4. Added a regression test that exercises forwarded namespace aliasing and asserts:
   - canonical callee name: `crypto.ctEq`
   - tag: `gate.crypto.ct_eq`
   - argument roles: `left_secret`, `right_secret`

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/security_map.rs`
  - `compiler/sec4-core/tests/security_map.rs`
- Output:
  - consistent metadata tagging for forwarded `crypto` namespace calls.
- Constraint:
  - this is alias/canonicalization-only; it does not change semantic type enforcement for `crypto.ctEq`.

## Failure modes and diagnostics

- If namespace tagging regresses, the security-map test fails because forwarded `c.ctEq(...)` remains unresolved and lacks the expected gate tag/roles.

## Example usage

```ut
fn cryptoNs() { crypto }

fn same(a: Secret<String>, b: Secret<String>) -> Bool {
  let c = cryptoNs();
  crypto.ctEq(a, b)
}
```

Security metadata now preserves gate visibility even when namespace values are forwarded through helper functions.

## Tradeoffs and next steps

- Tradeoff: namespace allowlist remains explicit; every new stdlib namespace still needs intentional registration.
- Next:
  - continue closing alias gaps for additional future namespaces so security-map tagging remains complete under indirection.
