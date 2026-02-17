# M38-S148 HTTP Max-Concurrency C-Bin Fixture Helper Introduction

## What it is

M38-S148 introduces a canonical fixture builder for max-concurrency runtime e2e tests.

## Why it exists

Project scaffolding, manifest/source writes, build invocation, and binary assertions were duplicated per test.

## How it works

- Added `build_c_bin_fixture(...)` helper.
- Helper takes fixture id, package name, source, and fixture label.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

- Apply fixture helper on queue paths (`M38-S149`).
