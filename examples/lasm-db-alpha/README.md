# lasm-db-alpha

Larger alpha example for validating LASM server mode with real disk-backed DB behavior.

## What this example proves

- `sec4 run --backend lasm` can persist DB records to disk (`records.log`).
- DB writes survive process restarts when `--db-base` (or `SEC4_RT_LASM_DB_BASE`) is set.
- `db.queryOne` style behavior returns the latest matching record.
- You can inspect full persisted state through `/db/records`.

## Files

- `src/main.ut`:
  - schema-tagged DB routes (`DbExecResponse`, `DbExecTxResponse`, `DbQueryOneResponse`, `DbListRecordsResponse`)
  - comments explain each route and expected query parameters.
- `sec4.toml`: project manifest.
- `sec4.policy`: minimal policy for local alpha runs.

## Run

1. Validate:

```bash
cargo run -p sec4 -- check --path examples/lasm-db-alpha
```

2. Start LASM server with explicit DB base:

```bash
DB_BASE="$(pwd)/examples/lasm-db-alpha/.lasm-db"
cargo run -p sec4 -- run \
  --path examples/lasm-db-alpha \
  --backend lasm \
  --db-base "$DB_BASE" \
  --port 8080
```

## Test flow (manual)

1. Append non-transactional record:

```bash
curl -i -X POST \
  'http://127.0.0.1:8080/db/exec?template=SELECT%201&params=alpha&db=1'
```

2. Append transactional record:

```bash
curl -i -X POST \
  'http://127.0.0.1:8080/db/exec-tx?template=SELECT%201&params=alpha&db=1&tx=9'
```

3. Query latest matching record:

```bash
curl -i \
  'http://127.0.0.1:8080/db/query-one?template=SELECT%201&params=alpha&db=1&row_schema=7'
```

4. List all persisted records:

```bash
curl -i 'http://127.0.0.1:8080/db/records'
```

5. Inspect persisted DB file:

```bash
cat "$DB_BASE/records.log"
```

## Expected shape

- `/db/exec` and `/db/exec-tx` return JSON with `recordId`, `op`, `db`, `template`, `params`, `tx`.
- `/db/query-one` returns latest matching `record` plus deterministic `row` text.
- `/db/records` returns `{ "ok": true, "count": N, "records": [...] }`.

## Notes

- `template` query parameter is required for `db` schema routes.
- Without `--db-base`/`SEC4_RT_LASM_DB_BASE`, records stay in-process only.
