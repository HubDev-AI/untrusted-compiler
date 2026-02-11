# 71 Benchmarking and Comparison Spec (Post-Stability)

This chapter defines how AILang will be benchmarked once the language/runtime is stable and working end-to-end.

The goal is not to claim "compiler speed" (v0 uses a C + clang backend). The goal is to prove service-level outcomes that match AILang's design:

- secure defaults with low overhead
- predictable performance under load
- stable tail latency
- debuggability and deterministic reproduction

## 1. Benchmark principles

- Benchmark end-to-end service behavior, not isolated compiler internals.
- Keep service behavior identical across languages.
- Prioritize p99 and failure-mode behavior over peak microbench numbers.
- Publish raw data and scripts so others can reproduce results.

## 2. Comparison targets

Core comparison set:

1. Go (`net/http` + one common router, e.g. `chi` or `fiber`)
2. Node.js TypeScript (Fastify baseline)
3. Rust (`axum` baseline)
4. AILang (C + clang backend)

Optional:

- Minimal C server as theoretical floor
- .NET/Java if enterprise audience requires it

## 3. Benchmark layers (run in this order)

### Layer A: Hello HTTP baseline

Scenario:

- `GET /ping` -> small `ok` response

Metrics:

- throughput (req/s)
- p50/p95/p99 latency
- CPU and RSS
- optional: binary size, startup time

Purpose:

- runtime/framework overhead baseline

### Layer B: JSON + schema validation

Scenario:

- `POST /decode` with realistic 2-10KB JSON body
- schema decode + validation + small response
- include invalid inputs to measure rejection cost

Metrics:

- p50/p95/p99 under load
- rejection path cost
- memory behavior (alloc pressure / RSS)

Purpose:

- validate AILang schema-gate design in realistic traffic

### Layer C: Real backend workload

Service shapes:

1. Read-heavy: `GET /users/:id` -> validate id -> DB read -> JSON
2. Write + validation: `POST /users` -> schema decode -> DB write -> JSON
3. Fanout (optional early, required later): `GET /enrich/:id` -> DB read + outbound HTTP call -> merge -> JSON

Metrics:

- throughput + p95/p99
- pool behavior and timeout behavior
- sustained memory profile
- backpressure/failure behavior

Purpose:

- measure the workload that actually matters for backend services

## 4. Fairness and credibility rules

- Same machine and environment for all runs.
- Same Postgres image, schema, indexes, and query text.
- Same pool sizes, timeouts, payload shapes, and response shapes.
- Same keep-alive behavior and release build mode.
- Warmup before measurement.
- Multiple runs per scenario (report median and spread).
- Use constant-rate testing (e.g., `wrk2`) and step-load to find saturation knee.

## 5. Load tools and run profile

Recommended primary tool:

- `wrk2` (constant rate)

Optional secondary tool:

- `k6` or `vegeta` for scripted workflows

Per scenario:

- warmup: 30s
- measure: 60-120s
- run several target rates and concurrency levels

## 6. Standard endpoint spec (v0 benchmark suite)

Base URL:

- `http://127.0.0.1:8080`

Endpoints:

1. `GET /ping` -> `200` `"ok"`
2. `POST /decode` -> schema validate + `200 {"ok":true,"id":"..."}`
3. `POST /users` -> schema validate + DB insert + `201 {"ok":true,"userId":"..."}`
4. `GET /users/:id` -> id validate + DB select + `200` JSON or `404` StdError

Error paths:

- invalid payload must return structured StdError (`400`)

## 7. Database baseline (Postgres)

For all implementations:

- shared schema and index definitions
- same `INSERT` and `SELECT` shapes
- fixed pool size (initial baseline: `32`)
- fixed statement timeout (initial baseline: `2s`)

## 8. AILang differentiator tracks

In addition to pure latency/throughput, benchmark these AILang-specific claims:

### 8.1 Security defaults overhead

- run with security defaults enabled (schema gates, typed sinks, URL safety)
- compare against other languages configured with equivalent protections
- measure extra cost at p99

### 8.2 Compile-time prevention proof

- include compile-fail fixtures for:
  - raw SQL concat
  - unescaped HTML sink
  - secret leak to logs/JSON
  - missing effect declaration
- publish outputs as part of report

### 8.3 Debuggability workflow

- capture failing request
- replay deterministically
- verify same error code and trace correlation
- track time-to-reproduce as an engineering KPI

## 9. Required outputs

Each run must write:

- raw load output
- machine-readable summary (`summary.json`)
- environment metadata (`env.json`)
- policy/audit metadata for AILang (`sec.audit` JSON + policy hash)

Recommended summary fields:

- implementation name + commit
- endpoint + load profile
- target/achieved rps
- p50/p95/p99/max latency
- error rate
- CPU avg, RSS avg
- binary size and startup time (optional)

## 10. Benchmark repository layout (target)

```text
benchmark-suite/
  spec/
    payloads/
    db/
  services/
    ailang/
    go/
    node/
    rust/
    c/                # optional floor reference
  load/
    wrk2/
  results/
    raw/
    summaries/
    plots/
  docker-compose.yml
  Makefile
```

## 11. Initial pass/fail targets (hardware-dependent, tune later)

These are starter targets and must be calibrated on actual benchmark hardware:

- `GET /ping`: p99 < 10ms at baseline target load
- `POST /decode`: p99 < 25ms at baseline target load
- DB read/write: no runaway memory, stable error rate under sustained load
- Replay demo reproduces deterministic error behavior

## 12. Communication framing

Public positioning line:

> AILang combines a security-typed backend frontend with a C/clang runtime path: high-level safety constraints with low-level performance characteristics.

This framing is valid only if measurements are reproducible and fair by the rules above.
