# 296 M32 Follow-up Slice: Runtime Header/Path Content Guards

This chapter documents the third post-closure runtime stabilization slice.

## What it is

Updated:
- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Added runtime guards:
- `headers.name` now enforces non-empty token-style header names (`[A-Za-z0-9-]+`).
- `headers.value` now rejects empty values and any CR/LF characters.
- `path.base` now requires absolute paths, rejects traversal markers (`..`), and rejects CR/LF.

## Why it exists

`M32-R1` removed zero-return stubs with deterministic handles, but still treated many string inputs as opaque. This slice adds concrete content validation for security-sensitive constructors that already receive inspectable strings at runtime.

## How it works internally

1. Added helper predicates in runtime C:
   - `sec4_rt_is_header_name_valid(...)`
   - `sec4_rt_is_header_value_valid(...)`
   - `sec4_rt_is_path_base_valid(...)`
2. Guarded constructor functions now return `0` on invalid input before hashing to opaque handles.
3. Valid inputs continue to map to deterministic non-zero handles via the existing hashing path.

## Validation

Added a direct runtime harness test (clang + runtime C):
- `c_bin_runtime_path_and_header_guards_when_clang_available`

The harness compiles and runs C code that calls `sec4_rt_headers_*` and `sec4_rt_path_base(...)` directly, asserting valid values succeed and invalid values fail.

Also revalidated core runtime asset expectations:
- `cargo test -p sec4-core --test c_backend`

## Trade-offs and next steps

- Trade-off:
  - URL content checks are not included yet because current `url.public/internal` bridge path carries opaque untrusted handles, not inspectable URL strings.
- Next:
  - Continue with `M32-R4` by introducing inspectable URL payload plumbing (or staged URL validator bridge) so SSRF-oriented runtime checks can be enforced concretely.
