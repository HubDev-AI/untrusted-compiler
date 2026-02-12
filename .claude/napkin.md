# Napkin

## Corrections
| Date | Source | What Went Wrong | What To Do Instead |
|------|--------|-----------------|--------------------|
| 2026-02-12 | self | Started session actions before confirming `.claude/napkin.md` existed and reading it. | Always check/create and read `.claude/napkin.md` first in-session. |
| 2026-02-12 | self | Assumed CLI/test hangs were caused by recent code changes; issue reproduced even on reverted state and simple local binaries. | Treat executable runtime as environment-level blocker first, but re-validate before locking the assumption. |
| 2026-02-12 | self | Treated runtime hang as persistent for the full session. | Re-run full runtime verification after environment hiccups; blockers can be transient. |
| 2026-02-12 | self | Deferred baseline verification too early due blocker assumption. | Keep full baseline verification (`cargo test`, `check`, `emit ast`, `build`) in the same slice once execution recovers. |
| 2026-02-12 | self | Inserted a new Rust test block inside an existing raw string, which produced cascading parser errors. | After patching large test files, immediately inspect the surrounding lines with `nl -ba` before running broad test suites. |

## User Preferences
- Keep strict milestone flow with docs updates and frequent commits (one commit per milestone slice).
- Add all design/spec changes into docs/book and keep roadmap aligned.
- Security-first language direction is mandatory.
- Avoid adding absolute local machine paths to committed docs.

## Patterns That Work
- Small vertical M4 slices with matching docs chapter updates reduce churn and keep progress reviewable.
- Deterministic `sec.audit` findings with sample evidence improve traceability for policy/security posture.
- After transient execution issues, retrying full verification in the same session often recovers and avoids false blockers.

## Patterns That Don't Work
- Launching multiple `cargo run` commands in parallel can cause lock contention/timeouts and noisy diagnostics.

## Domain Notes
- Current focus is M4 security foundation hardening before MIR/backend milestones.
- `sec.audit` and `security_map` are central artifacts for deterministic security posture and later editor tooling.

## Session Notes (2026-02-12)
- Repo currently has no `.trellis/` directory even though AGENTS references Trellis docs.
- Implemented and validated `sec audit --history-dir` with JSON-mode stdout contract preserved.
- Added `DNS_RESOLUTION_DISABLED` sec.audit posture rule with sample-call evidence (prod + `net.ssrf.resolve_dns=false`).
- Added `PUBLIC_EGRESS_NO_DOMAIN_POLICY` sec.audit posture rule (usage-gated on public-net sink calls) plus policy ingestion for `net.public.allowed_domains`/`blocked_domains`.
- Added `CSRF_PROTECTED_METHODS_INCOMPLETE` sec.audit posture rule with deterministic missing-method evidence and middleware sample-call context.
- Added `SYMLINK_POLICY_WEAK` sec.audit posture rule plus typed policy parsing/validation for `fs.forbid_symlinks`.
