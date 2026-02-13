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
   - `make db-up`
2. Apply schema:
   - `make db-schema`
3. Run benchmark profile (once service is implemented):
   - `make bench-ping IMPL=ailang`
   - `make bench-decode IMPL=ailang`
   - `make bench-users IMPL=ailang`
4. Stop DB:
   - `make db-down`

## Notes

- All services must implement identical endpoint behavior defined in `spec/endpoints.md`.
- Use constant-rate load for primary comparisons.
- Keep fairness controls from `docs/book/71-benchmarking-and-comparison-spec.md`.
