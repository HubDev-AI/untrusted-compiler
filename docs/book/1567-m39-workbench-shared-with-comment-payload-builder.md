# M39: Workbench Shared `with-comment` Payload Builder

## What changed

The canonical LASM workbench request bridge now builds `task_params`, `comment_params`, and the combined `params` payload for public `with-comment` requests from one shared payload materialization step.

File:

- `compiler/sec4-cli/src/lasm_request_template.rs`

## Why this mattered

The public JSON bridge for:

- `POST /wb/tasks/with-comment`
- `POST /wb/tasks/with-comment-tx`

already parsed the request body only once, but it still rebuilt the same task/comment fragments multiple times:

1. once for combined `params`,
2. again for `task_params`,
3. again for `comment_params`.

That was avoidable work on the canonical operator-facing path.

## What changed in the bridge

Added one shared helper:

- `build_lasm_workbench_tx_payload_params(...)`

It materializes:

- `task_params`
- `comment_params`
- `combined_params`

from the same parsed payload, generated ids, and generated timestamps.

Then `augment_lasm_workbench_query_params_from_body(...)` reuses that shared result instead of calling separate builders for the same request payload.

## What stayed the same

- public JSON contract is unchanged
- benchmark/query-string fallback contract is unchanged
- `task_params`, `comment_params`, and combined `params` remain deterministic
- `POST /wb/tasks/:id/comments` still uses its own route-specific helper

## Validation

Validated with:

```bash
cargo build -p sec4
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
```

`smoke-public.sh` still passes on:

- create task
- create task with one-statement route
- create task with explicit tx route
- add comment
- invalid input failures

## Tradeoff

This is not a benchmark miracle by itself. It is the right cleanup:

- less repeated serialization work,
- less repeated field extraction,
- same public contract.

That keeps the canonical workbench app cleaner while the next runtime/scaling gains continue to come from the Postgres transaction path itself.
