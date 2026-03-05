# M39 - Multi-File Build and Run Pipeline Lock

## What Was Added

Extended CLI integration coverage to prove multi-file modules run through real compile and runtime paths, not only static checks.

File:

1. `compiler/sec4-cli/tests/commands.rs`

New tests:

1. `build_emit_c_succeeds_for_multi_file_module_project`
2. `run_command_oneshot_serves_multi_file_module_route_and_exits`

## Why

`M39-S2A` needed pipeline confidence beyond resolver-only checks. These tests lock that multi-file module resolution survives:

1. C backend code generation,
2. `sec4 run --oneshot` runtime execution,
3. HTTP response/header behavior from a route implemented in a separate module file.

## Validation

1. `cargo test -p sec4 --test commands build_emit_c_succeeds_for_multi_file_module_project`
2. `cargo test -p sec4 --test commands run_command_oneshot_serves_multi_file_module_route_and_exits`
3. `cargo test -p sec4 --test commands`
