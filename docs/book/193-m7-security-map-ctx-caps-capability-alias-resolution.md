# 193 M7 Slice: security_map ctx.caps Capability Alias Resolution

This chapter documents the M7 callable-normalization hardening for capability objects accessed through `ctx.caps.*`.

## What it is

Added member-path capability alias normalization so security-map call resolution recognizes capability values sourced via:
- `ctx.caps.db`
- `ctx.caps.net`
- `ctx.caps.internalNet` / `ctx.caps.internal_net`
- `ctx.caps.fs`
- `ctx.caps.secrets`

## Why it exists

Before this slice, capability alias seeding only handled function parameters typed as capability tokens (`DbCap`, `NetCap`, and so on). Calls routed through context capability bags could lose canonical callee names and tags:

```ailang
let repo = ctx.caps.db;
repo.exec(repo, raw); // previously not normalized to db.exec
```

That produced metadata gaps in both `security_map` and downstream `sec.audit` evidence.

## How it works internally

1. Introduced `capability_namespace_alias_for_member_path(...)` in `security_map`.
2. Applied that normalization in callable-target resolution paths:
   - `infer_callable_alias(...)`
   - `normalize_callable_forward_target(...)`
   - `normalize_callable_summary_target(...)`
3. Added regression coverage for `ctx.caps.db` alias use and verified canonicalized sink metadata (`db.exec` + roles + origin tags).

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/security_map.rs`
  - `compiler/ailang-core/tests/security_map.rs`
- Output:
  - canonical sink/gate metadata preserved when capability handles are sourced from context capability bags.
- Constraint:
  - this is syntactic member-path normalization, not full object-shape type analysis.

## Failure modes and diagnostics

- If alias normalization regresses, the `ctx.caps.db` test fails because callsites remain unresolved (`repo.exec`) and no longer emit `sink.sql.exec` tags with expected argument roles.

## Example usage

```ailang
fn writeUser(ctx: Ctx, raw: SqlQuery) {
  let dbCap = ctx.caps.db;
  dbCap.exec(dbCap, raw)
}
```

Security-map output now treats the call as canonical `db.exec`.

## Tradeoffs and next steps

- Tradeoff: capability-bag support is intentionally pattern-based (`*.caps.*`) to keep v0 simple and deterministic.
- Next:
  - extend the same normalization strategy to other structured capability carriers once those shapes are stabilized.
