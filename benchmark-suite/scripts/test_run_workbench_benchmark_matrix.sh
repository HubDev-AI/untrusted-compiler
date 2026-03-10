#!/usr/bin/env bash
set -euo pipefail

suite_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "$suite_dir/.." && pwd)"

tmp_root="$(mktemp -d "${repo_root}/.tmp-workbench-benchmark-matrix-test.XXXXXX")"
trap 'rm -rf "$tmp_root"' EXIT

service_dir="${tmp_root}/service"
bin_dir="${tmp_root}/bin"
mkdir -p "$service_dir" "$bin_dir"

cat >"${service_dir}/server.mjs" <<'EOF'
import http from 'node:http';

const port = Number(process.env.PORT || 18130);

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
        message: 'intentional benchmark preflight failure',
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
Running 1s test @ http://127.0.0.1:18130
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
      "impl": "sec4-lasm",
      "status": "implemented",
      "servicePath": "${service_rel}"
    },
    {
      "impl": "node",
      "status": "implemented",
      "servicePath": "${service_rel}"
    }
  ]
}
EOF

out_runs="${tmp_root}/runs.json"
out_compare="${tmp_root}/compare.json"
out_analysis="${tmp_root}/analysis.json"
out_report="${tmp_root}/report.md"
out_report_html="${tmp_root}/report.html"
wrk_marker="${tmp_root}/wrk2-called.log"

set +e
out="$(
  PATH="${bin_dir}:$PATH" \
    BENCH_REQUIRE_WRK2=1 \
    FAKE_WRK2_MARKER="$wrk_marker" \
    "${suite_dir}/scripts/run_workbench_benchmark_matrix.sh" \
      --matrix "$matrix_path" \
      --impls node \
      --endpoints wb-task-get \
      --port 18130 \
      --out-runs "$out_runs" \
      --out-compare "$out_compare" \
      --out-analysis "$out_analysis" \
      --out-report "$out_report" \
      --out-report-html "$out_report_html" \
      2>&1
)"
status=$?
set -e

if [ "$status" -eq 0 ]; then
  echo "expected workbench benchmark matrix to fail on broken endpoint preflight" >&2
  echo "$out" >&2
  exit 1
fi

if [ -f "$wrk_marker" ]; then
  echo "expected benchmark matrix to fail before invoking wrk2" >&2
  exit 1
fi

if [ ! -f "$out_runs" ]; then
  echo "expected benchmark matrix failure summary to be written" >&2
  exit 1
fi

reason="$(jq -r '.runs[0].reason // ""' "$out_runs")"
if ! grep -q 'preflight failed endpoint=wb-task-get' <<<"$reason"; then
  echo "missing preflight failure reason in benchmark runs summary" >&2
  exit 1
fi
if ! grep -q 'status=500' <<<"$reason"; then
  echo "missing HTTP status detail in benchmark preflight reason" >&2
  exit 1
fi

if ! jq -e '.totals.failed == 1 and .totals.passed == 0' "$out_runs" >/dev/null; then
  echo "unexpected benchmark failure totals" >&2
  exit 1
fi

relaxed_runs="${tmp_root}/runs-relaxed.json"
set +e
relaxed_out="$(
  PATH="${bin_dir}:$PATH" \
    BENCH_REQUIRE_WRK2=1 \
    FAKE_WRK2_MARKER="$wrk_marker" \
    "${suite_dir}/scripts/run_workbench_benchmark_matrix.sh" \
      --matrix "$matrix_path" \
      --impls node \
      --endpoints wb-task-get \
      --port 18132 \
      --fail-on-impl-failure 0 \
      --out-runs "$relaxed_runs" \
      --out-compare "$out_compare" \
      --out-analysis "$out_analysis" \
      --out-report "$out_report" \
      --out-report-html "$out_report_html" \
      2>&1
)"
relaxed_status=$?
set -e

if [ "$relaxed_status" -ne 0 ]; then
  echo "expected relaxed benchmark matrix mode to return success despite impl failure" >&2
  echo "$relaxed_out" >&2
  exit 1
fi
if ! jq -e '.failOnImplFailure == 0 and .totals.failed == 1' "$relaxed_runs" >/dev/null; then
  echo "expected relaxed benchmark summary to keep failed totals with failOnImplFailure=0" >&2
  exit 1
fi

auto_mode_compare_missing="${tmp_root}/workbench-lasm-mode-compare-repeats-missing.json"
auto_out="$(
  PATH="${bin_dir}:$PATH" \
    BENCH_REQUIRE_WRK2=1 \
    FAKE_WRK2_MARKER="$wrk_marker" \
    "${suite_dir}/scripts/run_workbench_benchmark_matrix.sh" \
      --dry-run \
      --matrix "$matrix_path" \
      --impls sec4-lasm \
      --endpoints wb-task-get \
      --lasm-mode auto \
      --lasm-mode-compare-repeats-file "$auto_mode_compare_missing" \
      --port 18134 \
      --out-runs "$out_runs" \
      --out-compare "$out_compare" \
      --out-analysis "$out_analysis" \
      --out-report "$out_report" \
      --out-report-html "$out_report_html" \
      2>&1
)"
if ! grep -q 'warning: LASM auto mode recommendation unavailable (missing mode-compare artifact); generating fresh mode-compare artifact' <<<"$auto_out"; then
  echo "expected auto mode missing-artifact generation warning in dry-run output" >&2
  exit 1
fi
if ! grep -q 'run: .*run_workbench_lasm_mode_compare.sh .*--out .*workbench-lasm-mode-compare-repeats-missing.json .*--dry-run' <<<"$auto_out"; then
  echo "expected auto mode dry-run to show delegated mode-compare generation command" >&2
  exit 1
fi
if ! grep -q 'warning: LASM auto mode recommendation unavailable (missing mode-compare artifact); falling back to single mode' <<<"$auto_out"; then
  echo "expected auto mode fallback warning in dry-run output" >&2
  exit 1
fi
if ! grep -q 'start: impl=sec4-lasm .* lasmMode=single ' <<<"$auto_out"; then
  echo "expected auto mode fallback to single mode start marker in dry-run output" >&2
  exit 1
fi

echo "run_workbench_benchmark_matrix test passed"
