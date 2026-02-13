# 322 M13 Slice: First Trend-Run Results Note

This chapter captures the first M13 trend-note entry and the current threshold decision state.

## Observation timestamp

- Date: February 13, 2026
- Source: local operator run in repository workspace

## Commands executed

```bash
benchmark-suite/scripts/preflight.sh --impls node
BENCH_DURATION=10s benchmark-suite/scripts/run_comparison_matrix.sh --impls sec4,node,go,rust --endpoints ping,decode
benchmark-suite/scripts/render_trend_note_entry.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  --date 2026-02-13 \
  --baseline-dir benchmark-suite/baselines \
  --out benchmark-suite/results/summaries/trend-note-entry.md
```

Observed output:
- `wrk2` was not available locally; runner used explicit `wrk` fallback mode.
- Cross-implementation live endpoint metrics were captured for `ping` and `decode`.
- Trend note entry was rendered from live compare matrix output.

## Endpoint signal status

- `ping`: live signal captured (leader: `go`, p99: `1.75ms`)
- `decode`: live signal captured (leader: `go`, p99: `2.04ms`)

## Threshold posture decision

Decision for this note:
- no threshold changes

Rationale:
- local run used non-constant fallback mode (`wrk`), so coverage/guard outcomes are marked `n/a`,
- threshold changes should be based on scheduled constant-rate (`wrk2`) evidence.

## Follow-up action

For next threshold decision:
1. download latest trend artifact package:
   - `benchmark-suite/scripts/fetch_trend_artifact.sh --repo HubDev-AI/untrusted-compiler`
2. verify run mode is constant-rate (`wrk2`) before applying coverage-based guard logic.
3. evaluate against current thresholds and baseline limits.
4. append decision (`keep`, `tighten`, or `relax`) with rubric justification.

## Note rendering helper

Use the trend-note renderer to create deterministic markdown entries from `compare-matrix.json`:

```bash
benchmark-suite/scripts/render_trend_note_entry.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  --date 2026-02-13 \
  --baseline-dir benchmark-suite/baselines
```

Then import the generated entry into this chapter without duplicate insertion:

```bash
benchmark-suite/scripts/import_trend_note_entry.sh \
  --entry benchmark-suite/results/summaries/trend-note-entry.md
```

One-command flow (fetch + import):

```bash
benchmark-suite/scripts/update_trend_note_from_ci.sh \
  --repo HubDev-AI/untrusted-compiler
```

## Trend Entry (2026-02-13)

- Source matrix: `benchmark-suite/results/summaries/compare-matrix.json`
- Endpoints: `ping,decode`
- Run mode: local fallback (`wrk`), non-constant-rate
- Generators: wrk

| Endpoint | Leader | p99 (ms) | Coverage (%) | Absolute Guard | Baseline Guard |
| --- | --- | ---: | ---: | --- | --- |
| ping | go | 1.75 | n/a | n/a | n/a |
| decode | go | 2.04 | n/a | n/a | n/a |

- Overall absolute guard status: n/a
- Overall baseline guard status: n/a
