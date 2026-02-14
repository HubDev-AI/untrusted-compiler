# 476 M18 Kickoff Brief Generator

This chapter introduces the first M18 implementation slice: a deterministic kickoff brief built from clean-clone rehearsal evidence.

## 1) What it is

Script:

- `scripts/generate-m18-kickoff-brief.sh`

Contract test:

- `scripts/test-generate-m18-kickoff-brief.sh`

## 2) Why it exists

After M17 closeout evidence exists, M18 needs a compact transition artifact that summarizes readiness and recommended next actions from the latest rehearsal report.

## 3) Inputs

- `--report <path>` (default: `build/operator-clean-clone-rehearsal-live/rehearsal-report.json`)
- `--output <path>` (default markdown output path)
- `--format <markdown|json>`

## 4) Behavior

The generator validates report shape and derives:

- `overall`,
- `failedStep`,
- step/friction counts,
- deterministic recommendations.

Recommendation mode:

- PASS + zero friction: prioritize M18 scope matrix and keep rehearsal cadence.
- FAIL or friction present: block expansion and focus on failed-step remediation.

## 5) Outputs

- JSON mode: machine-readable kickoff summary.
- Markdown mode: `M18 Kickoff Brief` with summary + recommendations.

## 6) Verification

- `scripts/test-generate-m18-kickoff-brief.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
