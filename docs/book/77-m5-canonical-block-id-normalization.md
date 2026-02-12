# 77 M5 Slice: Canonical Block ID Normalization

This chapter documents the next M5 refinement: canonical MIR block id remapping after CFG lowering.

## Scope delivered
- MIR now applies a deterministic block-id normalization pass after lowering.
- All terminator targets (`goto`, `branch`, `switch`) are remapped consistently.
- Textual and JSON MIR outputs now use canonical display-order block ids.

## What changed
- Added `canonicalize_block_ids` in `mir.rs`.
- `lower_function` now runs CFG lowering first, then canonical block-id remapping.
- The remap keeps block order unchanged while reindexing ids to a compact `bb0..bbN` sequence.

## Why this matters
- MIR output is easier to read and compare across slices.
- Block IDs no longer depend on internal reservation order details.
- Backend consumers get stable, compact block-id spaces with explicit edge remapping.

## Tests updated
- MIR tests and golden fixtures were updated to reflect canonicalized block ids for:
  - statement-level continuation CFG cases
  - nested CFG lowering cases

## Tradeoffs
- Canonicalization is currently order-based (render order), not reachability/CFG-topology optimized.
- This is a presentation/stability normalization pass, not an optimization pass.

## Next step
- Introduce explicit temporary locals/value slots for branch-produced values where backend passes require value materialization.
