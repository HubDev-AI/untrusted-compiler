# 1513 M39 Slice: Local Postgres Artifact Ignore Rules

## What changed

1. Updated `.gitignore` with local infra artifact rules:
   - `infra/local-postgres/.env`
   - `infra/local-postgres/data/`

## Why

Local Postgres benchmark/infra runs generate these files as operational artifacts. Keeping them tracked as unignored untracked files creates persistent branch noise and raises accidental-stage risk during normal implementation flow.

## Validation

1. Verified local workspace status after ignore update:
   - `git status --short` shows only intended tracked edits and no local-postgres artifact noise.
