# 488 M19 Runtime Hardening Runner

This chapter documents M19-S4: the first executed M19 slice from selector output.

## 1) What changed

- Updated M19 selector recommendation IDs to `M19-S4-*` track IDs:
  - `M19-S4-runtime-hardening`
  - `M19-S4-release-hardening`
  - `M19-S4-editor-expansion`
  - `M19-S4-stabilization-remediation`
- Added runtime slice runner:
  - `scripts/run-m19-runtime-hardening.sh`
- Added runner contract test:
  - `scripts/test-run-m19-runtime-hardening.sh`
- Wired naming-lock + closure-gate enforcement:
  - `M19-D` in `scripts/check-milestone-closure.sh`
  - `scripts/test-check-milestone-closure.sh`
  - `.github/workflows/naming-lock.yml`

## 2) Why it matters

M19 already had planning artifacts (kickoff, matrix, selector), but no enforced execution entrypoint. This slice turns the selector recommendation into executable runtime-hardening work with deterministic guardrails.

## 3) How it works

- `run-m19-runtime-hardening.sh` validates selector shape and requires:
  - `selectedTrack == runtime`
  - recommendation ID prefix `M19-S4-runtime-`
- In dry-run mode it returns deterministic JSON/text with the command plan.
- In execution mode it runs:
  - runtime-smoke bundle validation,
  - runtime HTTP coverage contract test,
  - strict milestone closure check.

## 4) Verification

- `scripts/test-select-m19-next-slice.sh`
- `scripts/test-run-m19-runtime-hardening.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## 5) Tradeoff

M19-S4 executes only the runtime-selected slice. Release/editor execution runners remain separate future slices to keep execution deterministic and closure-gate evidence simple.
