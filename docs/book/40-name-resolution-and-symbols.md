# 40 Name Resolution and Symbols

This chapter documents the M2 name resolution pass in `compiler/sec4-core/src/semantic.rs`.

## What it is

A semantic pass that builds symbol catalogs for types and functions, then validates references in type annotations and expressions.

## Why it exists

Parsing alone cannot detect unknown identifiers, unknown types, or duplicate declarations. Name resolution is the first semantic integrity gate.

## How it works internally

- Builds a catalog of:
  - primitive and generic built-in types
  - user `struct` and `enum` declarations
  - function signatures (parameter types + return type)
- Checks duplicates for top-level declarations.
- Resolves every `TypeExpr` against known symbols.
- Resolves local expression identifiers against function scope.

## Inputs/outputs and constraints

- Input: parsed AST (`Program`) from M1 parser.
- Output: success or structured diagnostics.
- Constraint: current scope model is function-local; module/import resolution is not implemented yet.

## Failure modes and diagnostics

- `N3001`: unknown type
- `N3002`: duplicate declaration (type/function/parameter)
- `N3003`: unknown identifier
- `N3004`: generic arity misuse or non-generic type with type arguments

## Example usage

```bash
cargo run -p sec4 -- check --path examples/hello
```

Semantic diagnostics now run as part of `check` and `build`.

## Tradeoffs and next steps

- Tradeoff: no module scope or import graph yet.
- Tradeoff: declaration collection is single-file and intentionally strict.
- Next: add module/import resolution and cross-file symbol tables in the next M2 extension.
