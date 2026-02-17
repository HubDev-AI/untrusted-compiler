# M38-S164 HTTP Max-Concurrency Multi-Response Read Helper Introduction

## What it is

M38-S164 adds canonical two/three stream response-read helpers for contention tests.

## Why it exists

To avoid repeated inline response collection and keep failure envelopes consistent.

## How it works

- `read_two_http_responses(...)` and `read_three_http_responses(...)` wrap repeated stream read paths.
- Queue, late, and burst tests now share this surface.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
