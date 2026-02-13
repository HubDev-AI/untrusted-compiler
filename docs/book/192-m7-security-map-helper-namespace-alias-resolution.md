# 192 M7 Slice: security_map Helper Namespace Alias Resolution

This chapter documents the M7 alias-resolution extension for forwarded helper namespaces.

## What it is

Extended `security_map` namespace canonicalization so forwarded aliases for these stdlib namespaces are recognized:
- `sql`
- `json`
- `headers`
- `cookie`

## Why it exists

`security_map` already tagged direct calls like `sql.q(...)` or `json.encode(...)`, but forwarded namespace values could lose canonical names and tags:

```ailang
fn sqlNs() { sql }
let s = sqlNs();
s.q("SELECT 1", 1); // previously untagged
```

That made audit metadata incomplete under normal helper indirection patterns.

## How it works internally

1. Added the helper namespaces to `is_tagged_call_namespace(...)`.
2. Existing callable-forward normalization now accepts forwarded values rooted at those namespaces.
3. Added regression coverage that forwards each namespace through a helper function and verifies canonical callee + tag + arg roles for:
   - `sql.q`
   - `json.encode`
   - `headers.value`
   - `cookie.build`

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/security_map.rs`
  - `compiler/ailang-core/tests/security_map.rs`
- Output:
  - stable tagging and arg-role metadata for forwarded helper namespace calls.
- Constraint:
  - this is normalization metadata only; semantic type enforcement remains unchanged.

## Failure modes and diagnostics

- Regression appears as missing canonical call entries (for example `sqlRef.q` instead of `sql.q`) and missing tags like `gate.sql.parameterize` in `security_map` tests.

## Example usage

```ailang
fn jsonNs() { json }

fn encode(v: Int) {
  let j = jsonNs();
  j.encode("MySchema", v)
}
```

This call now keeps sink tagging in `security_map` even through namespace forwarding.

## Tradeoffs and next steps

- Tradeoff: namespace support remains explicit and curated to avoid accidental over-tagging.
- Next:
  - keep expanding namespace-forward coverage as new stdlib namespaces are introduced.
