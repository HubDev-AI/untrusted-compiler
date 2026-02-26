# Workbench: Feature-Rich Cross-Backend Benchmark

This folder defines a prompt-first workflow for generating and benchmarking equivalent real-DB backends across implementations.

## Purpose

1. Keep one canonical app contract.
2. Generate equivalent services for each backend.
3. Benchmark all of them under identical load/profile settings.

## Layout

1. `spec/feature-app-v1.md`:
   - canonical API, DB schema usage, deterministic envelopes.
2. `prompts/generate-feature-app-v1.md`:
   - generation prompt used for each backend target.
3. `matrix.backends.json`:
   - implementation targets and expected service roots.

## Workflow

1. Freeze contract (`spec/feature-app-v1.md`).
2. Generate per-backend services from `prompts/generate-feature-app-v1.md`.
3. Validate parity against contract.
4. Run existing benchmark-suite matrix runners.

## Current implementation lanes

1. `sec4-lasm-workbench` - `implemented-alpha`
2. `node-workbench` - `implemented-alpha`
3. `go-workbench` - `implemented-alpha`
4. `rust-workbench` - `implemented-alpha`
5. `sec4-workbench` - `implemented-alpha`

## Constraints

1. No placeholder/stub behavior in DB paths.
2. Mutating endpoints require auth.
3. Error/success envelope contract must be deterministic across all implementations.
