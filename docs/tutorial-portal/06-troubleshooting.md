# 06 - Troubleshooting

Use this as a fast triage page for the most common failures.

## CLI and build

### `sec4` not found

- Run commands with explicit binary:
  - `./target/debug/sec4 ...`
- Or export `target/debug` into `PATH`.

### `sec4 check` fails after fresh clone

- Rebuild:
  - `cargo build -p sec4 -p sec4audit-language-server`
- Re-run:
  - `./target/debug/sec4 check --path <project>`

## Zed plugin

### Language server not launching

1. `which sec4audit-language-server`
2. `scripts/manage-zed-extension-local.sh status`
3. reinstall extension:
   - `scripts/manage-zed-extension-local.sh update`

### Plugin regression after local changes

- rollback:
  - `scripts/manage-zed-extension-local.sh rollback`

## LASM + DB runtime

### `db adapter postgres requires ... DSN`

- source local env:
  - `set -a; if [ -f infra/local-postgres/.runtime.env ]; then source infra/local-postgres/.runtime.env; else source infra/local-postgres/.env; fi; set +a`
- or pass explicit flag:
  - `--db-postgres-dsn 'postgres://...'`

### `records.log` missing

- check active adapter:
  - `records` adapter writes `records.log`
  - `sqlite` writes `records.sqlite3`
  - `postgres` persists in DB (no local records file)

### Port already in use

- choose another:
  - `--port 18080`

## Benchmark failures

### Preflight fails due missing toolchain

- run:
  - `make -C benchmark-suite preflight`
- install missing dependency indicated by output.

### `wrk2` unavailable

- install wrapper:
  - `make -C benchmark-suite wrk2-install`
- export:
  - `export BENCH_WRK2_BIN="$PWD/benchmark-suite/bin/wrk2"`

### Benchmark run fails on one implementation lane

- rerun one lane first:
  - `make -C benchmark-suite bench-profile IMPL=<impl> ENDPOINT=<endpoint>`
- then rerun matrix.

## Useful health commands

```bash
git status --short
./target/debug/sec4 --help | head
scripts/test-alpha-implementation-fast.sh
scripts/check-zed-extension-operator-readiness.sh
```
