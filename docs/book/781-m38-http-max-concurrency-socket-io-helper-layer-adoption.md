# M38-S154 HTTP Max-Concurrency Socket I/O Helper-Layer Adoption

## What it is

M38-S154 moves contention socket I/O setup and read/write paths behind canonical helpers.

## Why it exists

To keep I/O failure messages and timeout behavior stable across all contention branches.

## How it works

- `set_stream_read_timeout(...)` owns timeout wiring.
- `write_http_request(...)` and `write_http_trailing_noise(...)` own request/noise writes.
- `read_http_response(...)` owns deterministic response collection.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
