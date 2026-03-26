# M39: Resource Command Verification And Doc Sync

## What it is

This slice verifies and tightens the post-alpha `resource` authoring surface that landed on `feature/resource-improvements`.

It covers three operator-facing commands:

- `sec4 describe`
- `sec4 migrate`
- `sec4 openapi`

and aligns them with the existing LASM resource runtime behavior for:

- `@unique`
- `@optional`
- typed defaults
- success-envelope responses

## Why it exists

The branch already added real functionality, but the integration pass found three correctness gaps:

1. the new commands were loading parse-only ASTs and could emit output for semantically invalid resources
2. generated DDL did not fully reflect `@unique`, `@optional`, or typed defaults
3. generated OpenAPI output described simplified row payloads instead of the actual runtime success envelopes

That combination is risky because it makes the authoring/contract surface look more complete than it really is.

## How it works internally

### Semantic analysis is now mandatory for resource command output

`sec4 describe`, `sec4 migrate`, and `sec4 openapi` now call `analyze_entry(...)` instead of `parse_entry_ast(...)`.

That means the same semantic gates used by `sec4 check` apply before the commands print any derived output. Invalid resource declarations now fail before route descriptions, DDL, or OpenAPI are emitted.

### DDL generation now follows resource field semantics

The CLI DDL path now includes:

- `UNIQUE` for `@unique`
- nullable omission for `@optional`
- typed default rendering for integer and boolean defaults
- adapter-specific timestamp defaults:
  - Postgres: `DEFAULT NOW()`
  - SQLite: `DEFAULT CURRENT_TIMESTAMP`

The runtime-side SQLite table helper was also updated so runtime-created tables and `sec4 migrate` output do not drift on required/default field behavior.

### OpenAPI output now matches the runtime contract

`sec4 openapi` now emits:

- success-envelope responses with:
  - `ok`
  - `status`
  - `traceId`
  - `timeMs`
  - `data`
- nullable field schemas for `@optional`
- create schemas that require only non-optional, non-default, writable fields
- update schemas with PATCH-like semantics (no required fields)
- list responses that describe:
  - `data.items`
  - `data.count`
  - `data.limit`
  - `data.offset`
- query parameters for:
  - `limit`
  - `offset`
  - `sort`
  - `order`
  - known field filters

## Inputs, outputs, and constraints

### Inputs

- a valid sec4 project with `sec4.toml`
- resource declarations in the resolved module graph
- optional adapter selection for `sec4 migrate`

### Outputs

- `sec4 describe`: human-readable field and route summary
- `sec4 migrate`: SQL `CREATE TABLE IF NOT EXISTS ...`
- `sec4 openapi`: OpenAPI 3.0 JSON

### Constraints

- resource output is still bound to the current LASM runtime contract:
  - update/delete remain `POST /:id/update` and `POST /:id/delete`
  - Postgres string-like fields currently map to `TEXT` in generated DDL for runtime compatibility
- the commands are not syntax-only utilities anymore; semantic validity is required

## Failure modes and diagnostics

The key change is that invalid resources now fail with semantic diagnostics before derived output is printed.

Example:

- `@primary` plus `@optional` now fails `sec4 migrate` with `error[E5006]`

This keeps the operator-facing commands aligned with `sec4 check` instead of letting invalid schemas silently produce misleading output.

## Example usage

```bash
sec4 describe --path ./myapp
sec4 migrate --path ./myapp --adapter postgres
sec4 openapi --path ./myapp
```

For:

```ut
resource Profile {
  id: Uuid @primary @auto,
  email: Email @unique,
  bio: String @optional,
  score: Int @default(0),
}
```

the verified behavior now includes:

- `describe` prints `@unique` and `@optional`
- `migrate` emits `UNIQUE`, omits `NOT NULL` on `bio`, and renders `DEFAULT 0`
- `openapi` marks `bio` nullable and keeps it out of `Create.required`

## Verification

Focused verification for this slice:

```bash
cargo test -p sec4-core --test golden_parser parser_fixtures_match_golden_output
cargo test -p sec4-core --test golden_semantic semantic_fixtures_match_golden_output
cargo test -p sec4 --test commands migrate_command_renders_unique_optional_and_defaults
cargo test -p sec4 --test commands migrate_command_fails_for_semantically_invalid_resource
cargo test -p sec4 --test commands describe_command_includes_unique_optional_and_route_shapes
cargo test -p sec4 --test commands openapi_command_respects_optional_fields_and_success_envelopes
```

## Tradeoffs and next steps

Tradeoffs:

- OpenAPI now reflects the current runtime contract instead of a cleaner hypothetical REST surface
- Postgres DDL keeps string-like fields on `TEXT` to match the runtime's prepared-statement path, even though richer native SQL types might look better on paper

Next steps:

1. decide whether resource response/error schemas should be promoted into reusable OpenAPI components
2. decide whether future REST-method support (`PUT` / `DELETE`) should be introduced as a route contract change or a parallel compatibility mode
3. keep the AI-facing docs (`ai-codegen-guide`, prompt template, Path B positioning) aligned whenever resource route shapes change
