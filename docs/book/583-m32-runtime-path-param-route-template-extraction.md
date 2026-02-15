# 583 M32 Follow-up Slice: Runtime Path Params from Route Templates

This chapter documents the sixth post-closure runtime stabilization slice.

## What it is

Updated:
- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Key runtime routing/path-param changes:
- Added route-template matcher that supports `:param` segments.
- Updated HTTP route matching and `Allow`-method collection to use template matching instead of exact path equality.
- Added path-param extraction against the matched route template and request route path.
- Stored matched template and normalized route path in request state for deterministic handler-time extraction.

## Why it exists

Before this slice, `req.pathParam` used fallback behavior and route matching required exact path strings. That blocked realistic handler behavior for routes like `/users/:id`.

## How it works internally

1. `sec4_rt_path_pattern_matches(pattern, path)` compares path segments and treats `:name` as wildcard segment.
2. `sec4_rt_extract_path_param(pattern, path, name, out, out_size)` extracts the concrete segment for the requested param name.
3. HTTP request handling now stores:
   - the route path (without query suffix)
   - the matched route template string
4. `sec4_rt_req_path_param("id")` uses matched template + route path to return tracked extracted values.

## Validation

Validated with oneshot HTTP runtime tests:
- `c_bin_http_runtime_serves_health_route_in_oneshot_mode_when_clang_available`
- `c_bin_http_runtime_matches_parameterized_route_in_oneshot_mode_when_clang_available`

The parameterized-route test asserts both successful route dispatch and non-fallback path-param extraction by comparing `req.pathParam("id")` vs missing `req.query("id")`.

## Trade-offs and next steps

- Trade-off:
  - v0 matcher supports segment params (`:id`) but not wildcard/splat patterns.
- Next:
  - continue runtime de-stub slices for remaining non-trivial runtime behavior gaps under the current M32 stabilization loop.
