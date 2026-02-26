# 1533 M39 Slice: Workbench Alpha Wire-Format Parity Across Backends

This slice aligns workbench request payload parsing across `node`, `go`, and `rust` lanes with the sec4 lane alpha wire format.

## What changed

1. `node-workbench` runtime parsing:
   - added support for `params`, `task_params`, and `comment_params` JSON-array query payloads,
   - preserved fallback support for prior explicit scalar query params.
2. `go-workbench` runtime parsing:
   - added JSON-array parsing helpers and int/string coercion for the same query payload keys,
   - added route-task-id consistency guard for comment payloads.
3. `rust-workbench` runtime parsing:
   - added JSON-array parsing helpers and structured input parsers (`TaskInput`, `TaskWithCommentInput`, `CommentInput`, `ListInput`),
   - preserved fallback support for explicit scalar query params.
4. Smoke harness updates:
   - node/go/rust smoke scripts now execute create/list flows through the shared alpha wire-format query payload keys.
5. Spec/prompt docs update:
   - `benchmark-suite/workbench/spec/feature-app-v1.md` now documents the alpha wire-format profile explicitly,
   - `benchmark-suite/workbench/prompts/generate-feature-app-v1.md` now requires that wire-format support in generated implementations.

## Why

Without a shared request wire format, each backend required backend-specific benchmark payload generators, which undermines parity and increases harness complexity. This slice establishes one deterministic alpha request shape across all current workbench implementations.

## Validation

1. `make -C benchmark-suite workbench-smoke` (`passed=5 failed=0 skipped=0`)
