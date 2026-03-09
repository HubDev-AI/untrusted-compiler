#!/usr/bin/env bash
set -euo pipefail

suite_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "$suite_dir/.." && pwd)"

tmp_root="$(mktemp -d "${repo_root}/.tmp-workbench-step-matrix-test.XXXXXX")"
tmp_mode_compare="$(mktemp)"
trap 'rm -rf "$tmp_root"; rm -f "$tmp_mode_compare"' EXIT

service_dir="${tmp_root}/service"
bin_dir="${tmp_root}/bin"
mkdir -p "$service_dir" "$bin_dir"

cat >"${service_dir}/server.mjs" <<'EOF'
import http from 'node:http';

const port = Number(process.env.PORT || 18131);

function writeJson(res, status, body) {
  res.writeHead(status, { 'content-type': 'application/json; charset=utf-8' });
  res.end(JSON.stringify(body));
}

function ok(res, body) {
  writeJson(res, 200, body);
}

function created(res, body) {
  writeJson(res, 201, body);
}

const server = http.createServer((req, res) => {
  const url = new URL(req.url || '/', `http://127.0.0.1:${port}`);

  if (req.method === 'GET' && url.pathname === '/health') {
    res.writeHead(200, { 'content-type': 'text/plain; charset=utf-8' });
    res.end('ok');
    return;
  }

  if (req.method === 'POST' && url.pathname === '/wb/setup') {
    ok(res, { ok: true, data: { reset: true } });
    return;
  }

  if (req.method === 'POST' && url.pathname === '/wb/tasks') {
    created(res, { ok: true, data: { id: 'seed-task' } });
    return;
  }

  if (req.method === 'POST' && /^\/wb\/tasks\/[^/]+\/comments$/.test(url.pathname)) {
    created(res, { ok: true, data: { id: 'seed-comment' } });
    return;
  }

  if (req.method === 'GET' && /^\/wb\/tasks\/[^/]+$/.test(url.pathname)) {
    writeJson(res, 500, {
      ok: false,
      error: {
        code: 'BROKEN.ENDPOINT',
        message: 'intentional step preflight failure',
      },
    });
    return;
  }

  writeJson(res, 404, {
    ok: false,
    error: {
      code: 'ROUTE.NOT_FOUND',
      message: 'not found',
    },
  });
});

server.listen(port, '127.0.0.1');
EOF

cat >"${bin_dir}/wrk2" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

: "${FAKE_WRK2_MARKER:?FAKE_WRK2_MARKER is required}"
printf 'wrk2 invoked\n' >>"$FAKE_WRK2_MARKER"

cat <<'OUT'
Running 1s test @ http://127.0.0.1:18131
  4 threads and 64 connections
  Thread calibration: mean lat.: 1.00ms, rate sampling interval: 10ms
  Latency     1.00ms    2.00ms    3.00ms   90.00%
  Req/Sec     1.00k    10.00     1.20k    90.00%
  100 requests in 1.00s, 10.00KB read
Requests/sec: 100.00
  50.000%    1.00ms
  95.000%    2.00ms
  99.000%    3.00ms
OUT
EOF
chmod +x "${bin_dir}/wrk2"

service_rel="${service_dir#${repo_root}/}"
matrix_path="${tmp_root}/matrix.json"
cat >"$matrix_path" <<EOF
{
  "implementations": [
    {
      "impl": "node",
      "status": "implemented",
      "servicePath": "${service_rel}"
    }
  ]
}
EOF

cat >"$tmp_mode_compare" <<'EOF'
{
  "config": {
    "endpoints": ["wb-tasks-list"],
    "lasmDbAdapter": "sqlite"
  },
  "recommendation": {
    "mode": "proxy"
  }
}
EOF

out_runs="${tmp_root}/runs.json"
out_step_matrix="${tmp_root}/step-matrix.json"
wrk_marker="${tmp_root}/wrk2-called.log"

set +e
out="$(
  PATH="${bin_dir}:$PATH" \
    BENCH_REQUIRE_WRK2=1 \
    BENCH_STEP_RATES=100 \
    FAKE_WRK2_MARKER="$wrk_marker" \
    "${suite_dir}/scripts/run_workbench_step_matrix.sh" \
      --matrix "$matrix_path" \
      --impls node \
      --endpoints wb-task-get \
      --port 18131 \
      --out-runs "$out_runs" \
      --out-step-matrix "$out_step_matrix" \
      2>&1
)"
status=$?
set -e

if [ "$status" -eq 0 ]; then
  echo "expected workbench step matrix to fail on broken endpoint preflight" >&2
  echo "$out" >&2
  exit 1
fi

if [ -f "$wrk_marker" ]; then
  echo "expected workbench step matrix to fail before invoking wrk2" >&2
  exit 1
fi

if [ ! -f "$out_runs" ]; then
  echo "expected step matrix failure summary to be written" >&2
  exit 1
fi

reason="$(jq -r '.runs[0].reason // ""' "$out_runs")"
if ! grep -q 'preflight failed endpoint=wb-task-get' <<<"$reason"; then
  echo "missing preflight failure reason in step runs summary" >&2
  exit 1
fi
if ! grep -q 'status=500' <<<"$reason"; then
  echo "missing HTTP status detail in step preflight reason" >&2
  exit 1
fi

if ! jq -e '.totals.failed == 1 and .totals.passed == 0' "$out_runs" >/dev/null; then
  echo "unexpected step failure totals" >&2
  exit 1
fi
if ! jq -e '(.matrixPath | startswith("/") | not) and (.stepMatrixPath | startswith("/") | not)' "$out_runs" >/dev/null; then
  echo "expected step runs summary paths to be repo-relative" >&2
  exit 1
fi

relaxed_runs="${tmp_root}/runs-relaxed.json"
set +e
relaxed_out="$(
  PATH="${bin_dir}:$PATH" \
    BENCH_REQUIRE_WRK2=1 \
    BENCH_STEP_RATES=100 \
    FAKE_WRK2_MARKER="$wrk_marker" \
    "${suite_dir}/scripts/run_workbench_step_matrix.sh" \
      --matrix "$matrix_path" \
      --impls node \
      --endpoints wb-task-get \
      --port 18133 \
      --fail-on-impl-failure 0 \
      --out-runs "$relaxed_runs" \
      --out-step-matrix "$out_step_matrix" \
      2>&1
)"
relaxed_status=$?
set -e

if [ "$relaxed_status" -ne 0 ]; then
  echo "expected relaxed step matrix mode to return success despite impl failure" >&2
  echo "$relaxed_out" >&2
  exit 1
fi
if ! jq -e '.failOnImplFailure == 0 and .totals.failed == 1' "$relaxed_runs" >/dev/null; then
  echo "expected relaxed step summary to keep failed totals with failOnImplFailure=0" >&2
  exit 1
fi

auto_mismatch_out="$(
  "${suite_dir}/scripts/run_workbench_step_matrix.sh" \
    --dry-run \
    --impls sec4-lasm \
    --endpoints wb-task-get \
    --lasm-mode auto \
    --lasm-mode-compare-repeats-file "$tmp_mode_compare" \
    --port 18135 \
    2>&1
)"
if ! grep -q 'warning: LASM auto mode recommendation artifact workload does not match requested benchmark workload; falling back to single mode' <<<"$auto_mismatch_out"; then
  echo "missing LASM auto mode workload mismatch fallback warning in step matrix output" >&2
  exit 1
fi
if ! grep -q 'start: impl=sec4-lasm .* lasmMode=single ' <<<"$auto_mismatch_out"; then
  echo "expected LASM auto mode fallback to single mode in step matrix output" >&2
  exit 1
fi

echo "run_workbench_step_matrix test passed"
