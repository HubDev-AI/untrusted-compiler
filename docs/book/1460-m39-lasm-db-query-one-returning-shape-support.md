# M39: LASM db.queryOne RETURNING-shape support

## What changed

`db.queryOne` runtime shape checks now accept row-returning DML statements in addition to pure select-like forms.

Accepted shapes now include:

- `SELECT`, `WITH`, `VALUES`, `TABLE`
- `INSERT`, `UPDATE`, `DELETE`, `MERGE` when the SQL contains `RETURNING`

This is implemented in the shared Postgres query-shape helper and used by both Postgres and SQLite `queryOne` paths.

## Runtime behavior

- Non-row-returning statements still fail deterministically.
- Row-returning DML (`... RETURNING`) now executes and materializes `row`/`rowObject` payloads through the existing `queryOne` response path.
- The DB runtime classifier now maps the new row-shape diagnostic text to deterministic validation envelopes:
  - status: `400`
  - code: `DB.QUERY_ONE_INVALID`

## Validation coverage

Command integration coverage now includes:

- SQLite `queryOne` with `INSERT ... RETURNING` success path.
- Postgres `queryOne` with `INSERT ... RETURNING` success path.
- Deterministic non-row-returning rejection assertions updated to the new row-shape diagnostic text.
