# 113 M7 Slice: Cookie Builder Intrinsic Bridge

This chapter documents the next M7 bootstrap slice: adding typed cookie construction helper support with runtime bridging and analysis hardening for namespace-shadowing aliases.

## Cookie Builder Helper

### What it is

Added intrinsic support for:
- `cookie.build` / `cookie_build`

Lowered runtime ABI stub:
- `ailang_rt_cookie_build`

Also added alias-resolution hardening in semantic and security-map analysis.

### Why it exists

The v0 stdlib surface includes typed cookie construction so response code can pass a `Cookie` value into `res.addCookie(...)` instead of raw string assembly. Without this bridge, that canonical flow cannot compile through `c-bin`.

### How it works internally

- Added semantic intrinsic entry:
  - `cookie.build` returns `Cookie`
- Added `cookie` to intrinsic namespace recognition.
- Added C rewrite mapping:
  - `cookie.build` / `cookie_build` -> `ailang_rt_cookie_build`
- Added runtime C declaration/definition for `ailang_rt_cookie_build`.
- Added security-map tagging and arg-role metadata:
  - `cookie.build` -> `gate.cookie.build`
  - arg roles: `name`, `value`
- Updated header/cookie integration fixtures to use `let cookie = cookie.build(...)` and pass that into `res.addCookie(cookie)`.

### Alias-resolution stability hardening

While integrating `cookie.build`, a namespace-shadowing pattern (`let cookie = cookie.build(...)`) exposed recursive alias expansion in callable name resolution:

- before: `cookie.build` could expand to `cookie.build.build...` and never converge
- now: alias resolution includes:
  - iteration cap (`64` steps),
  - recursive-suffix growth guard (`candidate.starts_with(name + ".")`)

This protection is applied in both semantic analysis and `security_map` call resolution.

### Inputs, outputs, and constraints

- Input: cookie helper calls in bootstrap service code.
- Output: generated C calls to `ailang_rt_cookie_build(...)` and `ailang_rt_set_cookie(...)`.
- Constraints:
  - runtime cookie behavior is still stubbed in M7.
  - this slice validates compile/link coverage and analysis stability.

### Failure modes and diagnostics

- Misspelled helper names fail semantic/C compile phases.
- Header/cookie sink checks remain active; untrusted/secret flow violations still report sink diagnostics for `res.addCookie`.

### Example usage

```ailang
fn configure() effects { net } -> Int {
  let cookie = cookie.build(1, 2);
  res.addCookie(cookie);
  0
}
```

### Tradeoffs and next steps

- This closes typed-cookie API surface coverage with minimal runtime complexity.
- Next steps are behavior-level cookie policy semantics (flags, same-site, secure/httpOnly defaults) in runtime-focused milestones.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - runtime ABI assertions include `ailang_rt_cookie_build`
  - header/cookie rewrite test now asserts typed cookie-build flow
- `compiler/ailang-cli/tests/json_output.rs`
  - header/cookie `c-bin` integration fixture uses `cookie.build(...)`
  - generated C assertions include `ailang_rt_cookie_build(...)`
