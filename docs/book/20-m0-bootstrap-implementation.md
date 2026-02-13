# 20 M0 Bootstrap Implementation Notes

This chapter documents what was implemented in M0 and how it currently works.

## Workspace and Module Skeleton

### What it is
A canonical repo layout and Rust workspace split into `sec4-core` and `sec4-cli`.

### Why it exists
To establish stable project boundaries before parser/type/backend work begins.

### How it works internally
- Root workspace (`Cargo.toml`) defines shared crate management.
- `sec4-core` contains reusable compiler logic.
- `sec4-cli` delegates command behavior to `sec4-core` functions.

### Inputs, outputs, and constraints
- Input: repo filesystem with expected directory structure.
- Output: compilable workspace with runnable `sec4` binary.
- Constraint: M0 does not include full compilation pipeline.

### Failure modes and diagnostics
- Missing files/dirs are surfaced by command-level diagnostics.

### Example usage
```bash
cargo run -p sec4 -- check --path examples/hello
```

### Tradeoffs and next steps
- Tradeoff: intentionally shallow functionality in exchange for stable foundation.
- Next: add parser and AST in M1.

## Diagnostics Framework

### What it is
A structured diagnostics model with severity, code, message, span, and notes.

### Why it exists
Stable error contracts are needed early for testability and tooling integration.

### How it works internally
- `Diagnostic` stores structured fields.
- `render_plain` supports golden tests.
- `render_color` provides ANSI colored terminal output.

### Inputs, outputs, and constraints
- Input: validation errors from manifest/project checks.
- Output: deterministic diagnostic text.
- Constraint: no source snippet rendering yet.

### Failure modes and diagnostics
- Invalid project states generate stable codes (`Mxxxx`, `Cxxxx`).

### Example usage
Diagnostics are emitted automatically on invalid manifest/entry path.

### Tradeoffs and next steps
- Tradeoff: simple formatter, no snippet context.
- Next: add source-line snippets in later milestones.

## Manifest and Build Validation

### What it is
M0 parser/validator for `sec4.toml` and entry file metadata.

### Why it exists
Compiler commands need a deterministic project contract before language compilation exists.

### How it works internally
- Parse TOML into `ManifestFile` (`package`, optional `build`).
- Validate required fields.
- Validate entry file existence and `.ut` extension.
- `build` writes lockfile stub.

### Inputs, outputs, and constraints
- Input: `<project>/sec4.toml`, expected entry file.
- Output: `Manifest` object or diagnostics.
- Constraint: lockfile is a stub, not dependency resolution.

### Failure modes and diagnostics
- Missing manifest (`M0001`)
- Invalid TOML (`M0002`)
- Empty package fields (`M0003`, `M0004`)
- Missing/invalid entry (`M0101`, `M0102`)
- Lockfile write failure (`M0201`)

### Example usage
```bash
cargo run -p sec4 -- build --path examples/hello
```

### Tradeoffs and next steps
- Tradeoff: build validates structure only, no code generation.
- Next: connect parser outputs in M1-M2.

## Golden Test Harness (Initial)

### What it is
Fixture-driven tests for manifest parsing and diagnostic rendering.

### Why it exists
Prevent regressions in output contracts and ensure deterministic diagnostics.

### How it works internally
- Fixtures under `compiler/sec4-core/tests/fixtures/manifest`.
- Test iterates `.toml` files and compares output to `.golden` files.

### Inputs, outputs, and constraints
- Input: fixture TOML and expected golden text.
- Output: pass/fail on exact match.
- Constraint: currently focused on manifest diagnostics only.

### Failure modes and diagnostics
- Missing golden files fail tests.
- Rendering changes break tests until golden is intentionally updated.

### Example usage
```bash
cargo test -p sec4-core
```

### Tradeoffs and next steps
- Tradeoff: narrow scope in M0.
- Next: extend harness to parser/type diagnostics in M1-M2.
