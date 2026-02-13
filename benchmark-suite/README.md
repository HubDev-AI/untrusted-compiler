# Benchmark Suite (M10 Scaffold)

This directory is the reproducible benchmark harness scaffold for M10.

## Goal

Measure end-to-end service behavior across identical implementations:
- AILang
- Go
- Node.js (TypeScript)
- Rust
- optional C floor reference

## Layout

- `spec/`: endpoint contract, payloads, and DB schema
- `services/`: per-language service implementations (to be added)
- `load/wrk2/`: load scripts
- `results/`: raw outputs, summaries, plots
- `docker-compose.yml`: shared Postgres for DB endpoints
- `Makefile`: benchmark orchestration entrypoints

## Initial workflow

1. Start DB:
   - `make -C benchmark-suite db-up`
2. Apply schema:
   - `make -C benchmark-suite db-schema`
3. Run benchmark profile (once service is implemented):
   - `make -C benchmark-suite bench-profile IMPL=ailang ENDPOINT=ping`
   - `make -C benchmark-suite bench-profile IMPL=ailang ENDPOINT=decode`
   - `make -C benchmark-suite bench-profile IMPL=ailang ENDPOINT=users-post`
4. Convert raw wrk2 output to summary JSON:
   - `make -C benchmark-suite summarize IMPL=ailang`
5. Bundle summaries + env into one report:
   - `make -C benchmark-suite report IMPL=ailang`
6. Build endpoint comparison from report bundles:
   - `make -C benchmark-suite compare`
7. Build all-endpoint comparison matrix:
   - `make -C benchmark-suite compare-matrix`
8. Validate benchmark helper scripts:
   - `make -C benchmark-suite test-scripts`
9. Stop DB:
   - `make -C benchmark-suite db-down`

## Notes

- All services must implement identical endpoint behavior defined in `spec/endpoints.md`.
- Use constant-rate load for primary comparisons.
- Keep fairness controls from `docs/book/71-benchmarking-and-comparison-spec.md`.
- `services/node`, `services/go`, and `services/rust` now include runnable baseline contract services; `services/ailang` and `services/c` remain scaffold placeholders.
