# 297 M32 Follow-up Slice: Runtime URL Content Guards

This chapter documents the fourth post-closure runtime stabilization slice.

## What it is

Updated:
- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Added runtime URL validation behavior:
- `url.public` now requires:
  - parseable URL with explicit scheme
  - `https` scheme
  - non-internal host
- `url.internal` now requires:
  - parseable `http`/`https` URL
  - internal/private host classification

## Runtime payload model change

To make URL runtime validation possible, runtime now tracks string payloads behind opaque handles:

- Added tracked value registry (`g_sec4_rt_tracked_values`) for string-backed untrusted values.
- `req.query` now attempts query extraction from request path and stores resolved value in tracked registry.
- `url.public` and `url.internal` resolve incoming handle back to tracked string value before applying URL checks.

A compatibility fallback remains:
- if a value is not tracked, URL functions preserve previous deterministic-handle behavior (no hard break for old bridge call paths).

## URL guard details (v0 runtime)

- URL parser extracts scheme and host (rejects missing host/userinfo-in-host segment).
- Internal host classifier treats as internal:
  - `localhost`
  - `.local` / `.internal` suffixes
  - private/link-local/loopback IPv4 ranges (`10/8`, `127/8`, `169.254/16`, `192.168/16`, `172.16/12`)
- Public URL validator rejects internal hosts and non-HTTPS.
- Internal URL validator requires host classified as internal.

## Validation

Added deterministic C runtime harness test:
- `c_bin_runtime_url_guards_when_clang_available`

Harness verifies:
- two valid public URLs produce non-zero distinct safe handles
- private/loopback URLs are rejected by `url.public`
- internal URLs are accepted by `url.internal`
- public URL is rejected by `url.internal`

Also revalidated:
- `c_bin_runtime_gate_handles_are_non_stub_when_clang_available`
- `cargo test -p sec4-core --test c_backend`

## Trade-offs and next steps

- Trade-off:
  - URL checks are heuristic/minimal (no DNS resolution, redirect-chain checks, or full policy profile enforcement yet).
- Next:
  - extend request-source extraction beyond query fallback and align URL runtime checks with policy-controlled SSRF settings.
