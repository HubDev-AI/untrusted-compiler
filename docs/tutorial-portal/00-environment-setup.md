# 00 - Environment Setup

This setup gives you a deterministic local environment for:

- `sec4` CLI development
- Zed plugin + language server usage
- LASM runs with real Postgres

## Prerequisites

- macOS or Linux
- Rust toolchain (`cargo`, `rustc`)
- `git`, `curl`
- Docker + Docker Compose plugin
- Zed editor (optional for CLI-only flow)

Optional but useful:

- `jq`
- `psql`
- `wrk2` (or use benchmark installer wrapper)

## Clone and build

```bash
git clone https://github.com/HubDev-AI/untrusted-compiler.git
cd untrusted-compiler

cargo build -p sec4 -p sec4audit-language-server
```

Sanity check:

```bash
./target/debug/sec4 --help | head
./target/debug/sec4audit-language-server --help | head
```

## Fast runtime sanity

```bash
scripts/smoke-sec4-run-hello-api.sh
```

If you only need a focused alpha implementation check:

```bash
scripts/test-alpha-implementation-fast.sh
```

## Bring up local Postgres infra

```bash
infra/local-postgres/scripts/up.sh
```

Load generated env (prefer runtime-resolved values):

```bash
set -a
if [ -f infra/local-postgres/.runtime.env ]; then
  source infra/local-postgres/.runtime.env
else
  source infra/local-postgres/.env
fi
set +a
```

Confirm DSN:

```bash
echo "$SEC4_RT_LASM_DB_POSTGRES_DSN"
```

Stop infra:

```bash
infra/local-postgres/scripts/down.sh
```

Reset infra data (destructive):

```bash
infra/local-postgres/scripts/reset.sh
```

## Next step

- Continue to `01-zed-plugin-workflow.md`.
