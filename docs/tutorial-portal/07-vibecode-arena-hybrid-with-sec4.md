# Vibecode Arena MVP: Hybrid Next.js + sec4 Backend

This guide is an execution handoff for building Vibecode Arena MVP with a **hybrid architecture**:

- Next.js + Supabase = product shell (auth, storage, UI, admin pages)
- sec4 (LASM backend) = internal voting/leaderboard engine

Use this layout for validation-first speed while still dogfooding sec4 where it adds the most value.

## 1) Why hybrid for this MVP

Use sec4 now for deterministic, high-integrity vote logic:

- matchup serving (`/vote/next`)
- vote write transaction + idempotency + anti-abuse checks
- ELO updates
- leaderboard query

Keep Next.js/Supabase for lower-risk integration-heavy pieces:

- X/Twitter OAuth + magic-link fallback
- submission form + media upload to Supabase Storage
- admin UI and moderation workflows

## 2) Where to find sec4 backend patterns in this repo

Canonical LASM + Postgres operator path:

- `docs/tutorial-portal/03-real-db-postgres-lasm.md`
- `benchmark-suite/services/sec4-lasm-workbench/README.md`
- `benchmark-suite/services/sec4-lasm-workbench/smoke.sh`

Real DB route patterns you should copy for Vibecode:

- `benchmark-suite/services/sec4-lasm-workbench/src/workbench/tasks.ut`
- `benchmark-suite/services/sec4-lasm-workbench/src/workbench/comments.ut`
- `benchmark-suite/services/sec4-lasm-workbench/src/workbench/common.ut`
- `benchmark-suite/services/sec4-lasm-workbench/src/workbench/setup.ut`

Focused DB-only examples:

- `examples/lasm-db-alpha/README.md`
- `examples/postgres-e2e/README.md`

## 3) Runtime command to run sec4 with Postgres

Use this shape for any Vibecode sec4 service:

```bash
cargo run -p sec4 -- run \
  --path <path-to-sec4-service> \
  --backend lasm \
  --db-adapter postgres \
  --db-postgres-dsn "$SEC4_RT_LASM_DB_POSTGRES_DSN" \
  --port 8787
```

For local DB bootstrap, use:

```bash
infra/local-postgres/scripts/up.sh
set -a; if [ -f infra/local-postgres/.runtime.env ]; then source infra/local-postgres/.runtime.env; else source infra/local-postgres/.env; fi; set +a
```

## 4) API boundary (recommended)

Public APIs remain in Next route handlers:

- `POST /api/submissions`
- `GET /api/vote/next?week=<slug>`
- `POST /api/vote`
- `GET /api/leaderboard?week=<slug>`

Route-handler mapping:

- Next validates auth/session/cookies and forwards to sec4 internal endpoints.
- sec4 performs DB transaction logic and returns deterministic JSON envelopes.

Recommended internal sec4 endpoints:

- `POST /internal/v1/vote/next`
- `POST /internal/v1/vote`
- `GET /internal/v1/leaderboard`
- `POST /internal/v1/admin/weeks/open-voting`
- `POST /internal/v1/admin/weeks/lock`

## 5) Security contract between Next and sec4

1. Next -> sec4 calls use a private shared token header (for example `X-Sec4-Internal-Token`).
2. sec4 internal routes reject missing/invalid token with deterministic 401.
3. Voter identity exposed to sec4 should be a derived session key + fingerprint hash, not raw PII.
4. Clients never call sec4 vote tables directly.

## 6) Data ownership split

Supabase-owned:

- auth identities
- profile metadata
- storage objects (GIF/MP4)
- admin-facing moderation metadata

sec4-owned (same Postgres DB, sec4 write path):

- matchup selection execution
- votes + idempotency enforcement
- ELO updates
- leaderboard query ordering

## 7) Build order for the other agent

1. Scaffold Next.js app and Supabase project.
2. Implement submission + moderation UI in Next/Supabase.
3. Implement sec4 internal vote engine service (LASM + Postgres) using workbench patterns.
4. Wire Next route handlers to sec4 internal endpoints.
5. Run one full flow:
   - builder submit -> admin approve -> open voting -> cast 10 votes -> leaderboard update.

## 8) Non-goals for this MVP integration

Do not implement yet:

- seasons / cumulative standings
- public profile/trophy system
- payments / boosts
- social feed/comments
- native apps
