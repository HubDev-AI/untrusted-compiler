# M39: LASM DB Record Signature Ref-Count Overflow Updates

Date: 2026-02-22  
Milestone: M39 (records-log fallback lookup optimization)

## What Changed

- Replaced dynamic-state signature index storage with ref-count tracking:
  - `db_record_signatures` now stores `HashMap<String, usize>` (`signature -> retained count`).
- Updated append/overflow path:
  - append increments the signature count,
  - capacity overflow now decrements/removes only the evicted records' signatures.
- Updated records-log fallback membership check:
  - `db.queryOne` records-log path now uses `contains_key` on the signature count map.
- Added duplicate-signature overflow regression coverage:
  - confirms signature membership remains present when one duplicate survives eviction.

## Why

The previous set-based index still rebuilt from all retained records whenever overflow happened. That made overflow handling O(n) even though only a small prefix was evicted.

## Result

- Overflow handling now performs incremental O(overflow) signature maintenance.
- `db.queryOne` records-log fallback stays O(1) on signature membership.
- Deterministic fallback semantics are preserved.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 append_db_record_enforces_capacity_by_dropping_oldest`
- `cargo test -p sec4 append_db_record_keeps_signature_when_duplicate_survives_overflow`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
