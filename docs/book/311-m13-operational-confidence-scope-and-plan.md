# 311 M13 Operational Confidence Scope and Plan

This chapter defines the post-M12 milestone scope and first execution slice for M13.

## Scope decision

M13 is locked to:
- performance consistency hardening,
- release promotion workflow enforceability,
- explain mapping coverage for priority policy/audit findings.

Deferred from M13:
- deep replay IO stubbing expansion,
- major editor UX expansion beyond current quick-fix baseline.

## M13 build tracks

1. Promotion workflow hardening
- document required release artifacts and identity checks,
- bind promotion flow to `release-alpha-gate` + `verify-release-promotion-inputs`.

2. Scheduled benchmark trend checks
- add a scoped scheduled benchmark workflow,
- publish deterministic artifacts and markdown summary.

3. Regression guardrails
- define baseline thresholds for key benchmark metrics,
- fail workflow deterministically when thresholds are breached.

4. Explain mapping expansion
- cover remaining high-signal policy/audit finding IDs.

## First execution slice (M13-S1)

Acceptance criteria:
- release promotion playbook chapter added and linked to runnable scripts/workflows,
- scheduled benchmark workflow added for scoped live run,
- at least one deterministic regression threshold check implemented.

Current status:
- done: release promotion playbook (`313`),
- done: scoped live workflow + threshold guard (`312`),
- pending: trend retention/baseline policy,
- pending: scoped live workflow expansion to one additional endpoint after stability window.

## Second execution slice (M13-S2 candidate)

Scope:
- codify first trend-run observations in book docs,
- document decode-threshold tuning rules tied to trend outcomes,
- define external publish handoff contract notes for release artifacts.

Acceptance criteria:
- trend-run results note chapter exists with explicit observations and action items,
- decode threshold tuning rubric is documented and linked from trend workflow docs,
- publish handoff notes chapter defines required manifest/artifact bindings for downstream tooling.

## Exit criteria

M13 is complete when:
- promotion flow is executable without ambiguous manual interpretation,
- scheduled benchmark trend data is produced consistently,
- regression thresholds are enforced in workflow checks,
- priority audit/policy finding IDs are explain-mapped.

## Working notes

This scope intentionally minimizes new runtime/compiler architecture risk while improving operational confidence for release and benchmark stability.
