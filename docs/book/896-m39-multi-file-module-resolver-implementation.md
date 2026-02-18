# M39 - Multi-File Module Resolver Implementation

## What Was Added

Implemented project-local multi-file module resolution for the compiler core.

Primary changes:

1. Added `compiler/sec4-core/src/project.rs` resolver.
2. Wired `parse_entry_ast` and `collect_allow_annotations` to compile all resolved modules instead of only the entry file.
3. Added CLI integration coverage for multi-file pass/fail contracts.

## Module Contract (v0.1)

`sec4` now supports explicit module imports with top-of-file directives:

- `use auth.validate;`

Resolution rules:

1. Source root defaults to `src/` when the entry is under `src/`.
2. Module path maps deterministically from file layout:
   - `src/auth/validate.ut` -> `auth.validate`
   - `src/foo/mod.ut` -> `foo`
3. `use` directives must appear before declarations.

## Deterministic Diagnostics

The resolver now emits stable errors for module-graph failures:

1. `M0303` - missing module import target.
2. `M0304` - ambiguous module target (multiple files resolve to the same module path).
3. `M0305` - cyclic module dependency.
4. `M0306` / `M0307` - invalid or misplaced `use` directives.

## Why

Alpha usability required real project composition beyond single-file entry programs. This slice delivers the minimum deterministic module graph needed to keep implementation velocity while preserving compiler clarity and strict diagnostics.

## Validation

Targeted checks executed:

1. `cargo test -p sec4-core project::tests::`
2. `cargo test -p sec4 --test commands check_succeeds_for_multi_file_module_project`
3. `cargo test -p sec4 --test commands check_fails_when_multi_file_module_is_missing`
4. `cargo test -p sec4 --test commands check_fails_when_multi_file_module_graph_has_cycle`
