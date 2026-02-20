# 1069 M39 Slice: LASM Alpha-Full Postgres Operator Flow

This slice updates the canonical `examples/lasm-alpha-full` operator flow to include explicit Postgres DB-adapter usage and the latest parameterized query semantics.

## What changed

1. Updated `examples/lasm-alpha-full/src/main.ut` top-level/operator comments to describe all supported DB adapter modes (`records-log`, `sqlite`, `postgres`).
2. Expanded `examples/lasm-alpha-full/README.md` with Postgres runtime startup instructions:
   - `--db-adapter postgres`
   - required `SEC4_RT_LASM_DB_POSTGRES_DSN`
3. Added operator-ready Postgres DB query demonstrations:
   - typed parameterized `db.queryOne` (`$N` + JSON-array params)
   - literal-preserving placeholder behavior (`'$1-literal'` remains literal)
   - deterministic placeholder-arity failure example (`$2` with one param)

## Why

The alpha example should demonstrate real LASM DB-client behavior that operators can run directly.

Including Postgres startup and parameterized query flows in the canonical example reduces ambiguity and makes DB adapter validation reproducible from one place.

## Validation

1. `cargo run -p sec4 -- check --path examples/lasm-alpha-full`
