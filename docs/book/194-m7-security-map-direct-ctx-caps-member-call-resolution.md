# 194 M7 Slice: security_map Direct ctx.caps Member-Call Resolution

This chapter documents the M7 canonicalization extension for direct capability-bag member calls.

## What it is

Extended security-map alias rewriting so direct call paths rooted in capability bags are canonicalized, for example:
- `ctx.caps.db.exec(...)` -> `db.exec(...)`
- `ctx.caps.net.get(...)` -> `httpClient.get(...)`
- `ctx.caps.secrets.get(...)` -> `secrets.get(...)`

## Why it exists

The previous slice handled local aliases (for example `let repo = ctx.caps.db; repo.exec(...)`), but direct member-call forms were still unresolved. That left sink/effect/capability tags missing in metadata when handlers called through `ctx.caps.*` without a local alias.

## How it works internally

1. Added `capability_member_path_alias(...)` to normalize capability-member paths into canonical stdlib namespaces.
2. Reused this normalization in:
   - callable alias inference
   - callable-forward summary normalization
   - alias-chain resolution loop (`resolve_alias_name`)
3. Added regression coverage for direct member-call form:
   - `ctx.caps.db.exec(ctx.caps.db, raw)`
   - asserting canonical callee/tag/arg-role/origin-edge output.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/security_map.rs`
  - `compiler/ailang-core/tests/security_map.rs`
- Output:
  - direct capability-bag calls now emit canonical security metadata.
- Constraint:
  - normalization is path-pattern driven and intentionally conservative.

## Failure modes and diagnostics

- If rewriting regresses, direct `ctx.caps.*` callsites remain non-canonical and fail security-map tests that expect tags like `sink.sql.exec`.

## Example usage

```ailang
fn write(ctx: Ctx, q: SqlQuery) {
  ctx.caps.db.exec(ctx.caps.db, q)
}
```

Security-map now records this as a canonical `db.exec` sink call.

## Tradeoffs and next steps

- Tradeoff: pattern-based normalization does not attempt arbitrary object-shape inference.
- Next:
  - extend normalization coverage for additional structured capability carriers as those patterns become stable.
