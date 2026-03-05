# Durable Postgres E2E (LASM) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a durable in-repo Postgres setup and a runnable LASM example that proves real DB connectivity and DB intrinsic behavior end-to-end.

**Architecture:** Use a hybrid layout: shared local infra under `infra/local-postgres/` plus a dedicated runnable example under `examples/postgres-e2e/`. Reuse existing LASM DB intrinsic response contracts (`DbExecResponse`, `DbExecTxResponse`, `DbQueryOneResponse`, `DbListRecordsResponse`) and wire runtime via `SEC4_RT_LASM_DB_ADAPTER=postgres` and DSN env.

**Tech Stack:** Docker Compose (Postgres 16), shell scripts, untrusted-compiler `.ut` example routes, `sec4` CLI (LASM backend), focused Rust integration test in `compiler/sec4-cli/tests/commands.rs`.

---

### Task 1: Shared local Postgres infra

**Files:**
- Create: `$REPO_ROOT/infra/local-postgres/docker-compose.yml`
- Create: `$REPO_ROOT/infra/local-postgres/.env.example`
- Create: `$REPO_ROOT/infra/local-postgres/scripts/up.sh`
- Create: `$REPO_ROOT/infra/local-postgres/scripts/down.sh`
- Create: `$REPO_ROOT/infra/local-postgres/scripts/reset.sh`

**Step 1: Write failing script contract checks (minimal shell validation)**

```bash
bash -n $REPO_ROOT/infra/local-postgres/scripts/up.sh
bash -n $REPO_ROOT/infra/local-postgres/scripts/down.sh
bash -n $REPO_ROOT/infra/local-postgres/scripts/reset.sh
```

Expected now: FAIL (files missing).

**Step 2: Add minimal compose + env template**

Create `docker-compose.yml` with service `postgres`:
- image `postgres:16-alpine`
- ports: `${PG_PORT:-5432}:5432`
- envs: `POSTGRES_DB`, `POSTGRES_USER`, `POSTGRES_PASSWORD`
- volume `./data:/var/lib/postgresql/data`
- healthcheck with `pg_isready`

Create `.env.example`:
- `PG_PORT=5432`
- `POSTGRES_DB=sec4_local`
- `POSTGRES_USER=sec4`
- `POSTGRES_PASSWORD=sec4dev`
- `SEC4_DB_ALPHA_DB_POSTGRES_DSN=postgres://sec4:sec4dev@127.0.0.1:5432/sec4_local?sslmode=disable`
  (or `SEC4_RT_LASM_DB_POSTGRES_DSN=...` as legacy-compatible alias)

**Step 3: Add scripts with fail-fast UX**

`up.sh`:
- verify `docker`/`docker compose`
- copy `.env.example` to `.env` when missing
- run `docker compose --env-file .env up -d`
- wait for healthy status
- print next commands

`down.sh`:
- `docker compose --env-file .env down`

`reset.sh`:
- `down` + remove `data` dir + `up`

**Step 4: Verify scripts parse and run help path**

Run:
```bash
bash -n $REPO_ROOT/infra/local-postgres/scripts/up.sh
bash -n $REPO_ROOT/infra/local-postgres/scripts/down.sh
bash -n $REPO_ROOT/infra/local-postgres/scripts/reset.sh
```
Expected: PASS.

**Step 5: Commit**

```bash
git add $REPO_ROOT/infra/local-postgres
git commit -m "M39: add durable local postgres infra for lasm e2e"
```

### Task 2: Postgres E2E example app

**Files:**
- Create: `$REPO_ROOT/examples/postgres-e2e/sec4.toml`
- Create: `$REPO_ROOT/examples/postgres-e2e/sec4.policy`
- Create: `$REPO_ROOT/examples/postgres-e2e/src/main.ut`
- Create: `$REPO_ROOT/examples/postgres-e2e/README.md`
- Create: `$REPO_ROOT/examples/postgres-e2e/scripts/smoke.sh`

**Step 1: Write failing validation command**

Run:
```bash
cargo run -p sec4 -- check --path $REPO_ROOT/examples/postgres-e2e
```
Expected now: FAIL (project missing).

**Step 2: Add manifest/policy + example routes**

Create `main.ut` with routes:
- `GET /health`
- `POST /db/exec` using template/params from query and `db.exec`
- `POST /db/exec-tx` using `db.tx` + `db.execTx`
- `GET /db/query-one` using `db.queryOne`
- `GET /db/records` returning `DbListRecordsResponse`

Use deterministic sample SQL in docs (for Postgres):
- create table + insert in exec/execTx examples
- select in queryOne example

**Step 3: Add smoke script**

`smoke.sh` should:
- load infra `.env`
- start `sec4 run --backend lasm --oneshot --path examples/postgres-e2e --port <free_port>`
- issue `curl` sequence for exec/execTx/queryOne/records
- assert response includes:
  - `HTTP/1.1 200`
  - `"adapter":"postgres"`
  - `"dbCache":{`

**Step 4: Verify example builds and smoke script parses**

Run:
```bash
cargo run -p sec4 -- check --path $REPO_ROOT/examples/postgres-e2e
bash -n $REPO_ROOT/examples/postgres-e2e/scripts/smoke.sh
```
Expected: PASS.

**Step 5: Commit**

```bash
git add $REPO_ROOT/examples/postgres-e2e
git commit -m "M39: add postgres e2e lasm example with smoke script"
```

### Task 3: Focused CLI integration verification

**Files:**
- Modify: `$REPO_ROOT/compiler/sec4-cli/tests/commands.rs`

**Step 1: Add failing integration test**

Add test `run_command_postgres_e2e_example_check_passes` that runs:
- `sec4 check --path examples/postgres-e2e`

Expected first run: FAIL until example exists and paths are wired.

**Step 2: Implement minimal assertion logic**

Assert:
- exit success
- no unexpected diagnostics

Optional second test (skip-aware):
- `run_command_postgres_e2e_smoke_script_parses`
- `bash -n examples/postgres-e2e/scripts/smoke.sh`

**Step 3: Run focused tests**

```bash
cargo test -p sec4 --test commands run_command_postgres_e2e_example_check_passes
```

Expected: PASS.

**Step 4: Run one manual real DB flow**

```bash
$REPO_ROOT/infra/local-postgres/scripts/up.sh
$REPO_ROOT/examples/postgres-e2e/scripts/smoke.sh
```

Expected: PASS, real Postgres-backed responses.

**Step 5: Commit**

```bash
git add $REPO_ROOT/compiler/sec4-cli/tests/commands.rs
git commit -m "M39: add focused command coverage for postgres e2e example"
```

### Task 4: Documentation + roadmap/book updates

**Files:**
- Modify: `$REPO_ROOT/docs/05-sec4-master-roadmap.md`
- Create: `$REPO_ROOT/docs/book/1407-m39-durable-postgres-e2e-infra-and-example.md`
- Modify: `$REPO_ROOT/docs/book/README.md`

**Step 1: Add roadmap bullet for durable Postgres E2E setup**

Document:
- shared infra path
- example path
- smoke path
- real DB proof criteria

**Step 2: Add book chapter**

Include:
- what changed
- why
- how to run end-to-end
- failure diagnostics

**Step 3: Update book index**

Add chapter entry to `/docs/book/README.md`.

**Step 4: Validate touched commands/docs**

Run:
```bash
cargo test -p sec4 --test commands run_command_postgres_e2e_example_check_passes
```

**Step 5: Commit**

```bash
git add $REPO_ROOT/docs/05-sec4-master-roadmap.md \
        $REPO_ROOT/docs/book/1407-m39-durable-postgres-e2e-infra-and-example.md \
        $REPO_ROOT/docs/book/README.md
git commit -m "M39: document durable postgres e2e infra and example"
```

### Task 5: PR and merge flow

**Files:**
- No source files; git/GitHub operations only.

**Step 1: Push branch**

```bash
git push -u origin codex/m39-postgres-e2e-design-doc
```

**Step 2: Open PR to `dev`**

Include summary + exact validation commands from tasks above.

**Step 3: Merge with squash**

```bash
gh pr merge <id> --squash --delete-branch
```

**Step 4: Confirm `dev` is clean and synced**

```bash
git status --short --branch
```

Expected: clean `dev...origin/dev`.

**Step 5: Record final status**

Post concise operator report:
- infra path
- example path
- smoke command
- pass/fail result
