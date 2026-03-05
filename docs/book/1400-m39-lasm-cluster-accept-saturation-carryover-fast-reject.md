# M39: LASM Cluster Accept Saturation Carryover Fast Reject

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `compiler/sec4-cli/src/lasm_cluster_accept_loop.rs` to carry a relay saturation signal across accept-loop batches.
- When a dispatch attempt observes sustained relay saturation, fallback scan work is now short-circuited in subsequent attempts until a dispatch succeeds.
- Successful primary or fallback dispatch resets saturation carryover immediately.

## Why

Under sustained overload, per-request fallback scans can burn CPU repeatedly while all relay queues are still full. Batch-local short-circuiting existed, but the signal was reset at each batch boundary, causing repeated scan work to reappear quickly.

## Result

- Overload path now rejects faster with less repeated fallback scanning while saturation persists.
- Recovery behavior is preserved: the first successful dispatch clears the carryover signal and normal routing resumes.
- Healthy-path dispatch semantics remain unchanged.

## Validation

- `rustfmt compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
