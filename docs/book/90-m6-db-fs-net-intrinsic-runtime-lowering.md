# 90 M6 Slice: DB/FS/Net Intrinsic Runtime Lowering

This chapter documents the next M6 vertical slice: lowering core IO intrinsics (`db.*`, `fs.*`, `httpClient.get*`) to runtime ABI symbols.

## IO Intrinsic Rewriting

### What it is

C emission now rewrites:
- `db.exec(...)` / `db.queryOne(...)`
- `fs.read(...)` / `fs.write(...)`
- `httpClient.get(...)` / `httpClient.getInternal(...)`

and underscore aliases to runtime symbols:
- `ailang_rt_db_exec`, `ailang_rt_db_query_one`
- `ailang_rt_fs_read`, `ailang_rt_fs_write`
- `ailang_rt_http_get`, `ailang_rt_http_get_internal`

Runtime ABI now includes stub declarations/definitions for each symbol.

### Why it exists

These APIs are central to backend work. This slice extends the C backend’s intrinsic-lowering coverage so capability/effect-checked IO call shapes compile end-to-end while real runtime adapters are still pending.

### How it works internally

- Added string-based replacements in `lower_c_expr(...)` for dotted and underscore intrinsic names.
- Added matching runtime stubs under `runtime/c/ailang_runtime.h/.c`.
- Existing expression rewrite path automatically applies lowering across statement and return contexts.

### Inputs, outputs, and constraints

- Input: MIR expressions with supported IO intrinsic names.
- Output: C-valid runtime calls and successful compile/link flow.
- Constraints:
  - stubs currently return placeholder values.
  - capability/effect semantic checks remain authoritative before backend emission.

### Failure modes and diagnostics

- Missing capability arguments or missing effect declarations still fail semantic checks (`E2003`, `E2001`).
- Unsupported intrinsic spellings remain unlowered and may fail C compile/link.

### Example usage

```ailang
fn ioOps(
  db: DbCap,
  fs: FsCap,
  net: NetCap,
  query: SqlQuery,
  path: PathSafe,
  url: PublicUrl
) effects { db.write, db.read, fs.read, fs.write, net } -> Int {
  db.exec(db, query);
  db.queryOne(db, query, 1);
  fs.read(fs, path);
  fs.write(fs, path, 1);
  httpClient.get(net, url);
  0
}
```

### Tradeoffs and next steps

- This slice prioritizes backend plumbing, not real DB/FS/network behavior.
- Runtime implementations remain no-op/placeholder.
- Next step: replace these stubs with adapter-backed runtime behavior in M7 HTTP/JSON and runtime bridge milestones.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - new `c_backend_rewrites_db_fs_and_net_intrinsics_to_runtime_symbols`
- `compiler/ailang-cli/tests/json_output.rs`
  - new `build_emit_c_bin_handles_db_fs_net_intrinsics_when_clang_available`
