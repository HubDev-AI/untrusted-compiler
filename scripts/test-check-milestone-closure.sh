#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/scripts" "$tmp/.github/workflows" "$tmp/benchmark-suite/results/summaries" "$tmp/docs/book"

cat > "$tmp/scripts/release-alpha-gate.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
scripts/check-milestone-closure.sh --fail-on-pending
SH
chmod +x "$tmp/scripts/release-alpha-gate.sh"
touch "$tmp/scripts/verify-release-promotion-inputs.sh"
touch "$tmp/scripts/generate-release-publish-manifest.sh"
touch "$tmp/scripts/verify-release-publish-manifest.sh"
touch "$tmp/scripts/test-check-m17-operator-handoff-readiness.sh"
touch "$tmp/scripts/test-run-m17-operator-bootstrap.sh"
touch "$tmp/scripts/test-print-m17-operator-troubleshooting-matrix.sh"
touch "$tmp/scripts/test-run-m17-operator-handoff-quickstart.sh"
touch "$tmp/scripts/test-run-m17-operator-handoff-ci-smoke.sh"
touch "$tmp/scripts/test-operator-handoff-workflow-contract.sh"
touch "$tmp/scripts/test-operator-handoff-workflow-contract-guard.sh"
touch "$tmp/scripts/test-inspect-m17-operator-handoff-artifacts.sh"
touch "$tmp/scripts/test-summarize-m17-operator-handoff-readiness.sh"
touch "$tmp/scripts/test-build-m17-operator-release-packet.sh"
touch "$tmp/scripts/test-check-m17-operator-handoff-playbook.sh"
touch "$tmp/scripts/test-run-m17-operator-clean-clone-rehearsal.sh"
touch "$tmp/scripts/test-generate-m18-kickoff-brief.sh"
touch "$tmp/scripts/test-build-m18-priority-matrix.sh"
touch "$tmp/scripts/test-select-m18-next-slice.sh"
touch "$tmp/scripts/test-m18-editor-contract-expansion.sh"
touch "$tmp/scripts/test-m18-release-publish-integrity.sh"
touch "$tmp/scripts/test-run-m18-runtime-track.sh"
touch "$tmp/scripts/test-build-m18-track-convergence-summary.sh"
touch "$tmp/scripts/test-build-m18-transition-handoff-packet.sh"
touch "$tmp/scripts/test-build-m18-closure-report.sh"
touch "$tmp/scripts/test-generate-m19-kickoff-brief.sh"
touch "$tmp/scripts/test-build-m19-priority-matrix.sh"
touch "$tmp/scripts/test-select-m19-next-slice.sh"
touch "$tmp/scripts/test-run-m19-runtime-hardening.sh"
touch "$tmp/scripts/test-build-m19-executed-slice-convergence-summary.sh"
touch "$tmp/scripts/test-build-m19-transition-handoff-packet.sh"
touch "$tmp/scripts/test-build-m19-closure-report.sh"
touch "$tmp/scripts/test-generate-m20-kickoff-brief.sh"
cat > "$tmp/scripts/test-replay-cli-json-contract.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo '"mockExecutionCounts"'
echo '"mockExecutionTraces"'
SH
chmod +x "$tmp/scripts/test-replay-cli-json-contract.sh"
cat > "$tmp/scripts/test-replay-cli-json-contract-guard.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "expected replay json contract failure when mockExecutionCounts key is missing"
echo "expected replay json contract failure when mockExecutionTraces key is missing"
SH
chmod +x "$tmp/scripts/test-replay-cli-json-contract-guard.sh"
cat > "$tmp/.github/workflows/alpha-release-gate.yml" <<'YAML'
name: Alpha Release Gate
on:
  workflow_dispatch:
  push:
    branches:
      - main
jobs:
  alpha-release-gate:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run alpha release gate script
        run: scripts/release-alpha-gate.sh
      - name: Verify release promotion inputs
        run: scripts/verify-release-promotion-inputs.sh build/release-alpha-gate/release-summary.txt
      - name: Generate release publish manifest
        run: scripts/generate-release-publish-manifest.sh
      - name: Verify release publish manifest
        run: scripts/verify-release-publish-manifest.sh build/release-alpha-gate/release-publish-manifest.json
      - name: Upload alpha release gate artifacts
        uses: actions/upload-artifact@v4
        with:
          name: alpha-release-gate-artifacts
          path: build/release-alpha-gate
YAML
cat > "$tmp/.github/workflows/release-contract-smoke.yml" <<'YAML'
name: Release Contract Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  release-contract-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Verify alpha release gate workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Verify alpha release gate workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
      - name: Verify release publish manifest verifier
        run: scripts/test-verify-release-publish-manifest.sh
      - name: Verify release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
YAML
cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate replay capture contract
        run: scripts/test-replay-capture-contract.sh
      - name: Validate replay capture compatibility
        run: scripts/test-replay-capture-compat.sh
      - name: Validate replay stub registry contract
        run: scripts/test-replay-stub-registry-contract.sh
      - name: Validate replay CLI json contract
        run: scripts/test-replay-cli-json-contract.sh
      - name: Validate replay CLI json contract guard behavior
        run: scripts/test-replay-cli-json-contract-guard.sh
      - name: Validate M16 runtime HTTP coverage contract
        run: scripts/test-m16-runtime-http-coverage.sh
      - name: Validate M16 runtime HTTP coverage guard behavior
        run: scripts/test-m16-runtime-http-coverage-guard.sh
      - name: Validate sec4 run hello-api smoke script contract
        run: scripts/test-smoke-sec4-run-hello-api-script-contract.sh
      - name: Validate sec4 run hello-api smoke script contract guard behavior
        run: scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh
      - name: Validate runtime-smoke workflow contract
        run: scripts/test-runtime-smoke-workflow-contract.sh
      - name: Validate runtime-smoke workflow contract guard behavior
        run: scripts/test-runtime-smoke-workflow-contract-guard.sh
      - name: Validate runtime-smoke artifacts checker
        run: scripts/test-check-runtime-smoke-artifacts.sh
      - name: Validate runtime-smoke branch index builder
        run: scripts/test-build-runtime-smoke-branch-index.sh
      - name: Validate runtime-smoke bundle checker
        run: scripts/test-check-runtime-smoke-bundle.sh
      - name: Validate M17 operator handoff readiness checker
        run: scripts/test-check-m17-operator-handoff-readiness.sh
      - name: Validate M17 operator bootstrap profile helper
        run: scripts/test-run-m17-operator-bootstrap.sh
      - name: Validate M17 operator troubleshooting matrix
        run: scripts/test-print-m17-operator-troubleshooting-matrix.sh
      - name: Validate M17 operator handoff quickstart
        run: scripts/test-run-m17-operator-handoff-quickstart.sh
      - name: Validate M17 operator handoff CI smoke wrapper
        run: scripts/test-run-m17-operator-handoff-ci-smoke.sh
      - name: Validate operator-handoff workflow contract
        run: scripts/test-operator-handoff-workflow-contract.sh
      - name: Validate operator-handoff workflow contract guard behavior
        run: scripts/test-operator-handoff-workflow-contract-guard.sh
      - name: Validate M17 operator handoff artifact inspector
        run: scripts/test-inspect-m17-operator-handoff-artifacts.sh
      - name: Validate M17 operator handoff readiness summary
        run: scripts/test-summarize-m17-operator-handoff-readiness.sh
      - name: Validate M17 operator release packet builder
        run: scripts/test-build-m17-operator-release-packet.sh
      - name: Validate M17 final handoff playbook checker
        run: scripts/test-check-m17-operator-handoff-playbook.sh
      - name: Validate M17 clean-clone rehearsal runner
        run: scripts/test-run-m17-operator-clean-clone-rehearsal.sh
      - name: Validate M18 kickoff brief generator
        run: scripts/test-generate-m18-kickoff-brief.sh
      - name: Validate M18 priority matrix artifact
        run: scripts/test-build-m18-priority-matrix.sh
      - name: Validate M18 next-slice selector
        run: scripts/test-select-m18-next-slice.sh
      - name: Validate M18 editor contract expansion
        run: scripts/test-m18-editor-contract-expansion.sh
      - name: Validate M18 release publish integrity contract expansion
        run: scripts/test-m18-release-publish-integrity.sh
      - name: Validate M18 runtime-track execution runner
        run: scripts/test-run-m18-runtime-track.sh
      - name: Validate M18 track convergence summary
        run: scripts/test-build-m18-track-convergence-summary.sh
      - name: Validate M18 transition handoff packet
        run: scripts/test-build-m18-transition-handoff-packet.sh
      - name: Validate M18 closure report
        run: scripts/test-build-m18-closure-report.sh
      - name: Validate M19 kickoff brief
        run: scripts/test-generate-m19-kickoff-brief.sh
      - name: Validate M19 priority matrix
        run: scripts/test-build-m19-priority-matrix.sh
      - name: Validate M19 next-slice selector
        run: scripts/test-select-m19-next-slice.sh
      - name: Validate M19 runtime hardening runner
        run: scripts/test-run-m19-runtime-hardening.sh
      - name: Validate M19 executed-slice convergence summary
        run: scripts/test-build-m19-executed-slice-convergence-summary.sh
      - name: Validate M19 transition handoff packet
        run: scripts/test-build-m19-transition-handoff-packet.sh
      - name: Validate M19 closure report
        run: scripts/test-build-m19-closure-report.sh
      - name: Validate M20 kickoff brief
        run: scripts/test-generate-m20-kickoff-brief.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate local path leak guard
        run: scripts/test-check-no-local-path-leaks.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
      - name: Validate zed grammar pin guard behavior
        run: scripts/test-zed-grammar-pin-guard.sh
YAML
cat > "$tmp/.github/workflows/benchmark-smoke.yml" <<'YAML'
name: Benchmark Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  benchmark-scripts:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run benchmark script smoke tests
        run: |
          scripts/test-benchmark-smoke-closure-gate.sh
          scripts/test-benchmark-smoke-closure-gate-guard.sh
          scripts/test-benchmark-cross-impl-workflow-contract.sh
          scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
          scripts/test-benchmark-trend-workflow-contract.sh
          scripts/test-benchmark-trend-workflow-contract-guard.sh
          scripts/test-check-milestone-closure.sh
          scripts/check-milestone-closure.sh --fail-on-pending
YAML

cat > "$tmp/.github/workflows/benchmark-trend.yml" <<'YAML'
name: Benchmark Trend
on:
  workflow_dispatch:
  schedule:
    - cron: '0 7 * * 1'
jobs:
  scoped-live-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json \
            --fail-on-warning
      - name: Check regression thresholds (ping)
        run: |
          benchmark-suite/scripts/check_regression_thresholds.sh \
            benchmark-suite/results/summaries/compare-matrix.json \
            --endpoint ping
      - name: Upload benchmark trend artifacts
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-trend-node-ping
          path: benchmark-suite/results
YAML

cat > "$tmp/.github/workflows/benchmark-cross-impl-evidence.yml" <<'YAML'
name: Benchmark Cross-Impl Evidence
on:
  workflow_dispatch:
jobs:
  cross-impl-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Run cross-impl benchmark matrix (ping+decode)
        run: |
          benchmark-suite/scripts/run_comparison_matrix.sh \
            --impls sec4,node,go,rust \
            --endpoints ping,decode
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json \
            --fail-on-warning
      - name: Upload cross-impl benchmark evidence
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-cross-impl-evidence
          path: benchmark-suite/results
YAML

cat > "$tmp/.github/workflows/runtime-smoke.yml" <<'YAML'
name: Runtime Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  runtime-smoke:
    runs-on: ubuntu-latest
    timeout-minutes: 20
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run sec4 hello-api operator smoke (default)
        run: scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke/default
      - name: Run sec4 hello-api operator smoke (max-body)
        run: scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir build/runtime-smoke/max-body
      - name: Validate runtime smoke bundle
        run: scripts/check-runtime-smoke-bundle.sh --artifacts-root build/runtime-smoke --index-path build/runtime-smoke/runtime-smoke-branch-index.json
      - name: Upload runtime smoke artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: runtime-smoke-artifacts
          path: build/runtime-smoke
YAML

cat > "$tmp/.github/workflows/operator-handoff-smoke.yml" <<'YAML'
name: Operator Handoff Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  operator-handoff-smoke:
    runs-on: ubuntu-latest
    timeout-minutes: 25
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run M17 operator handoff CI smoke wrapper
        run: scripts/run-m17-operator-handoff-ci-smoke.sh --artifacts-root build/operator-handoff-smoke
      - name: Upload operator handoff smoke artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: operator-handoff-smoke-artifacts
          path: build/operator-handoff-smoke
YAML

cat > "$tmp/benchmark-suite/results/summaries/compare-matrix.json" <<'JSON'
{
  "version": "0.1",
  "endpoints": [
    {
      "endpoint": "ping",
      "compared": [
        { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "go", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "node", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "rust", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
      ],
      "leader": { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
    }
  ]
}
JSON

cat > "$tmp/docs/book/322-m13-first-trend-run-results-note.md" <<'MD'
# Trend note

## Trend Entry (2026-02-13)
MD

audit_output="$("$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending)"
if printf '%s\n' "$audit_output" | rg -q -- "$tmp"; then
  echo "expected repo-relative evidence paths in closure audit output" >&2
  exit 1
fi

audit_json="$("$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json --fail-on-pending)"
if ! printf '%s\n' "$audit_json" | jq -e '.overall == "PASS" and .pendingCount == 0' >/dev/null; then
  echo "expected PASS json closure summary for passing fixture" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '
  [.gates[].gate] == [
    "M9-A","M9-B","M9-C","M9-D","M9-E","M9-F","M9-G","M9-H",
    "M10-A","M10-B","M10-C","M10-D",
    "M11-A","M12-A","M12-B",
    "M13-A","M13-B","M13-C","M13-D","M13-E","M13-F",
    "M14-A","M14-B","M14-C","M14-D",
    "M15-A","M16-A","M16-B","M16-C","M16-D","M16-E",
    "M17-A","M17-B","M17-C","M17-D","M17-E","M17-F","M17-G","M17-H","M17-I","M17-J","M17-K","M17-L","M18-A","M18-B","M18-C","M18-D","M18-E","M18-F","M18-G","M18-H","M18-I","M19-A","M19-B","M19-C","M19-D","M19-E","M19-F","M19-G","M20-A"
  ]
' >/dev/null; then
  echo "expected deterministic gate ordering in json closure output" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M9-H") != null' >/dev/null; then
  echo "expected json closure output to include M9-H gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M10-D") != null' >/dev/null; then
  echo "expected json closure output to include M10-D gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M11-A") != null' >/dev/null; then
  echo "expected json closure output to include M11-A gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M12-A") != null' >/dev/null; then
  echo "expected json closure output to include M12-A gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M12-B") != null' >/dev/null; then
  echo "expected json closure output to include M12-B gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M13-D") != null' >/dev/null; then
  echo "expected json closure output to include M13-D gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M13-E") != null' >/dev/null; then
  echo "expected json closure output to include M13-E gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M13-F") != null' >/dev/null; then
  echo "expected json closure output to include M13-F gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M14-A") != null' >/dev/null; then
  echo "expected json closure output to include M14-A gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M14-B") != null' >/dev/null; then
  echo "expected json closure output to include M14-B gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M14-C") != null' >/dev/null; then
  echo "expected json closure output to include M14-C gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M14-D") != null' >/dev/null; then
  echo "expected json closure output to include M14-D gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M15-A") != null' >/dev/null; then
  echo "expected json closure output to include M15-A gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M16-A") != null' >/dev/null; then
  echo "expected json closure output to include M16-A gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M16-B") != null' >/dev/null; then
  echo "expected json closure output to include M16-B gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M16-C") != null' >/dev/null; then
  echo "expected json closure output to include M16-C gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M16-D") != null' >/dev/null; then
  echo "expected json closure output to include M16-D gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M16-E") != null' >/dev/null; then
  echo "expected json closure output to include M16-E gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-A") != null' >/dev/null; then
  echo "expected json closure output to include M17-A gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-B") != null' >/dev/null; then
  echo "expected json closure output to include M17-B gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-C") != null' >/dev/null; then
  echo "expected json closure output to include M17-C gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-D") != null' >/dev/null; then
  echo "expected json closure output to include M17-D gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-E") != null' >/dev/null; then
  echo "expected json closure output to include M17-E gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-F") != null' >/dev/null; then
  echo "expected json closure output to include M17-F gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-G") != null' >/dev/null; then
  echo "expected json closure output to include M17-G gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-H") != null' >/dev/null; then
  echo "expected json closure output to include M17-H gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-I") != null' >/dev/null; then
  echo "expected json closure output to include M17-I gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-J") != null' >/dev/null; then
  echo "expected json closure output to include M17-J gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-K") != null' >/dev/null; then
  echo "expected json closure output to include M17-K gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M17-L") != null' >/dev/null; then
  echo "expected json closure output to include M17-L gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M18-A") != null' >/dev/null; then
  echo "expected json closure output to include M18-A gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M18-B") != null' >/dev/null; then
  echo "expected json closure output to include M18-B gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M18-C") != null' >/dev/null; then
  echo "expected json closure output to include M18-C gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M18-D") != null' >/dev/null; then
  echo "expected json closure output to include M18-D gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M18-E") != null' >/dev/null; then
  echo "expected json closure output to include M18-E gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M18-F") != null' >/dev/null; then
  echo "expected json closure output to include M18-F gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M18-G") != null' >/dev/null; then
  echo "expected json closure output to include M18-G gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M18-H") != null' >/dev/null; then
  echo "expected json closure output to include M18-H gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M18-I") != null' >/dev/null; then
  echo "expected json closure output to include M18-I gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M19-A") != null' >/dev/null; then
  echo "expected json closure output to include M19-A gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M19-B") != null' >/dev/null; then
  echo "expected json closure output to include M19-B gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M19-C") != null' >/dev/null; then
  echo "expected json closure output to include M19-C gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M19-D") != null' >/dev/null; then
  echo "expected json closure output to include M19-D gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M19-E") != null' >/dev/null; then
  echo "expected json closure output to include M19-E gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M19-F") != null' >/dev/null; then
  echo "expected json closure output to include M19-F gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M19-G") != null' >/dev/null; then
  echo "expected json closure output to include M19-G gate" >&2
  exit 1
fi
if ! printf '%s\n' "$audit_json" | jq -e '.gates | map(.gate) | index("M20-A") != null' >/dev/null; then
  echo "expected json closure output to include M20-A gate" >&2
  exit 1
fi
if printf '%s\n' "$audit_json" | rg -q -- "$tmp"; then
  echo "expected repo-relative evidence paths in json closure output" >&2
  exit 1
fi

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-check-m17-operator-handoff-readiness\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M17 operator handoff readiness checker step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-A")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-A to become pending when naming-lock workflow misses handoff readiness checker step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-run-m17-operator-bootstrap\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M17 operator bootstrap profile helper step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-B")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-B to become pending when naming-lock workflow misses bootstrap profile helper step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-print-m17-operator-troubleshooting-matrix\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M17 operator troubleshooting matrix step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-C")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-C to become pending when naming-lock workflow misses troubleshooting matrix step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-run-m17-operator-handoff-quickstart\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M17 operator handoff quickstart step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-D")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-D to become pending when naming-lock workflow misses handoff quickstart step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-run-m17-operator-handoff-ci-smoke\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M17 operator handoff CI smoke wrapper step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-E")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-E to become pending when naming-lock workflow misses CI smoke wrapper step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/operator-handoff-smoke.yml" "$tmp/.github/workflows/operator-handoff-smoke.base.yml"
awk '!/scripts\/run-m17-operator-handoff-ci-smoke\.sh --artifacts-root build\/operator-handoff-smoke/' "$tmp/.github/workflows/operator-handoff-smoke.base.yml" > "$tmp/.github/workflows/operator-handoff-smoke.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when operator-handoff workflow misses CI smoke wrapper run step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-F")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-F to become pending when operator-handoff workflow misses CI smoke wrapper run step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/operator-handoff-smoke.base.yml" "$tmp/.github/workflows/operator-handoff-smoke.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-operator-handoff-workflow-contract-guard\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses operator-handoff workflow contract guard step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-G")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-G to become pending when naming-lock workflow misses operator-handoff workflow contract guard step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-inspect-m17-operator-handoff-artifacts\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M17 operator handoff artifact inspector step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-H")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-H to become pending when naming-lock workflow misses artifact inspector step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-summarize-m17-operator-handoff-readiness\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M17 operator handoff readiness summary step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-I")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-I to become pending when naming-lock workflow misses readiness summary step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-build-m17-operator-release-packet\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M17 operator release packet builder step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-J")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-J to become pending when naming-lock workflow misses release packet builder step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-check-m17-operator-handoff-playbook\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M17 final handoff playbook checker step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-K")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-K to become pending when naming-lock workflow misses final handoff playbook checker step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-run-m17-operator-clean-clone-rehearsal\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M17 clean-clone rehearsal runner step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M17-L")).status == "PENDING"
' >/dev/null; then
  echo "expected M17-L to become pending when naming-lock workflow misses clean-clone rehearsal runner step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-generate-m18-kickoff-brief\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M18 kickoff brief generator step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M18-A")).status == "PENDING"
' >/dev/null; then
  echo "expected M18-A to become pending when naming-lock workflow misses kickoff brief generator step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-build-m18-priority-matrix\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M18 priority matrix artifact step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M18-B")).status == "PENDING"
' >/dev/null; then
  echo "expected M18-B to become pending when naming-lock workflow misses priority matrix artifact step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-select-m18-next-slice\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M18 next-slice selector step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M18-C")).status == "PENDING"
' >/dev/null; then
  echo "expected M18-C to become pending when naming-lock workflow misses next-slice selector step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-m18-editor-contract-expansion\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M18 editor contract expansion step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M18-D")).status == "PENDING"
' >/dev/null; then
  echo "expected M18-D to become pending when naming-lock workflow misses editor contract expansion step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-m18-release-publish-integrity\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M18 release publish integrity contract expansion step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M18-E")).status == "PENDING"
' >/dev/null; then
  echo "expected M18-E to become pending when naming-lock workflow misses release publish integrity contract expansion step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-run-m18-runtime-track\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M18 runtime-track execution runner step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M18-F")).status == "PENDING"
' >/dev/null; then
  echo "expected M18-F to become pending when naming-lock workflow misses runtime-track execution runner step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-build-m18-track-convergence-summary\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M18 track convergence summary step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M18-G")).status == "PENDING"
' >/dev/null; then
  echo "expected M18-G to become pending when naming-lock workflow misses track convergence summary step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-build-m18-transition-handoff-packet\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M18 transition handoff packet step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M18-H")).status == "PENDING"
' >/dev/null; then
  echo "expected M18-H to become pending when naming-lock workflow misses transition handoff packet step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-build-m18-closure-report\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M18 closure report step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M18-I")).status == "PENDING"
' >/dev/null; then
  echo "expected M18-I to become pending when naming-lock workflow misses closure report step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-generate-m19-kickoff-brief\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M19 kickoff brief step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M19-A")).status == "PENDING"
' >/dev/null; then
  echo "expected M19-A to become pending when naming-lock workflow misses kickoff brief step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-build-m19-priority-matrix\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M19 priority matrix step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M19-B")).status == "PENDING"
' >/dev/null; then
  echo "expected M19-B to become pending when naming-lock workflow misses priority matrix step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-select-m19-next-slice\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M19 next-slice selector step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M19-C")).status == "PENDING"
' >/dev/null; then
  echo "expected M19-C to become pending when naming-lock workflow misses next-slice selector step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-run-m19-runtime-hardening\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M19 runtime hardening runner step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M19-D")).status == "PENDING"
' >/dev/null; then
  echo "expected M19-D to become pending when naming-lock workflow misses runtime hardening runner step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-build-m19-executed-slice-convergence-summary\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M19 executed-slice convergence summary step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M19-E")).status == "PENDING"
' >/dev/null; then
  echo "expected M19-E to become pending when naming-lock workflow misses executed-slice convergence summary step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-build-m19-transition-handoff-packet\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M19 transition handoff packet step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M19-F")).status == "PENDING"
' >/dev/null; then
  echo "expected M19-F to become pending when naming-lock workflow misses transition handoff packet step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-build-m19-closure-report\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M19 closure report step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M19-G")).status == "PENDING"
' >/dev/null; then
  echo "expected M19-G to become pending when naming-lock workflow misses closure report step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cp "$tmp/.github/workflows/naming-lock.yml" "$tmp/.github/workflows/naming-lock.base.yml"
awk '!/scripts\/test-generate-m20-kickoff-brief\.sh/' "$tmp/.github/workflows/naming-lock.base.yml" > "$tmp/.github/workflows/naming-lock.yml"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses M20 kickoff brief step" >&2
  exit 1
fi

if ! "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json | jq -e '
  .overall == "PENDING"
  and (.gates[] | select(.gate == "M20-A")).status == "PENDING"
' >/dev/null; then
  echo "expected M20-A to become pending when naming-lock workflow misses kickoff brief step" >&2
  exit 1
fi

mv "$tmp/.github/workflows/naming-lock.base.yml" "$tmp/.github/workflows/naming-lock.yml"

cat > "$tmp/docs/book/322-m13-first-trend-run-results-note.md" <<'MD'
# Trend note
MD

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure without trend entry" >&2
  exit 1
fi

if pending_json="$("$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --format json --fail-on-pending 2>/dev/null)"; then
  echo "expected pending failure exit code for json closure output without trend entry" >&2
  exit 1
fi
if ! printf '%s\n' "$pending_json" | jq -e '.overall == "PENDING" and .pendingCount > 0' >/dev/null; then
  echo "expected pending json closure summary for failing fixture" >&2
  exit 1
fi

cat > "$tmp/docs/book/322-m13-first-trend-run-results-note.md" <<'MD'
# Trend note

## Trend Entry (2026-02-13)
MD

cat > "$tmp/benchmark-suite/results/summaries/compare-matrix.json" <<'JSON'
{
  "version": "0.1",
  "endpoints": [
    {
      "endpoint": "ping",
      "compared": [
        { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "go", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "node", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
      ],
      "leader": { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
    }
  ]
}
JSON

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure without per-endpoint rust impl coverage" >&2
  exit 1
fi

cat > "$tmp/benchmark-suite/results/summaries/compare-matrix.json" <<'JSON'
{
  "version": "0.1",
  "endpoints": [
    {
      "endpoint": "ping",
      "compared": [
        { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "go", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "node", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" },
        { "impl": "rust", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
      ],
      "leader": { "impl": "sec4", "endpoint": "ping", "targetRps": 1, "requestsPerSec": 1, "loadGenerator": "wrk2", "constantRate": true, "p99": "1ms" }
    }
  ]
}
JSON

cat > "$tmp/.github/workflows/benchmark-trend.yml" <<'YAML'
name: Benchmark Trend
on:
  workflow_dispatch:
jobs:
  scoped-live-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Check regression thresholds (ping)
        run: |
          benchmark-suite/scripts/check_regression_thresholds.sh \
            benchmark-suite/results/summaries/compare-matrix.json \
            --endpoint ping
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure without strict benchmark trend workflow quality gate" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/benchmark-trend.yml" <<'YAML'
name: Benchmark Trend
on:
  workflow_dispatch:
  schedule:
    - cron: '0 7 * * 1'
jobs:
  scoped-live-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json \
            --fail-on-warning
      - name: Check regression thresholds (ping)
        run: |
          benchmark-suite/scripts/check_regression_thresholds.sh \
            benchmark-suite/results/summaries/compare-matrix.json \
            --endpoint ping
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure without benchmark trend artifact upload step" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/benchmark-trend.yml" <<'YAML'
name: Benchmark Trend
on:
  workflow_dispatch:
  schedule:
    - cron: '0 7 * * 1'
jobs:
  scoped-live-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json \
            --fail-on-warning
      - name: Check regression thresholds (ping)
        run: |
          benchmark-suite/scripts/check_regression_thresholds.sh \
            benchmark-suite/results/summaries/compare-matrix.json \
            --endpoint ping
      - name: Upload benchmark trend artifacts
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-trend-node-ping
          path: benchmark-suite/results
YAML

cat > "$tmp/scripts/release-alpha-gate.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
scripts/check-milestone-closure.sh
SH
chmod +x "$tmp/scripts/release-alpha-gate.sh"

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when release gate omits strict closure flag" >&2
  exit 1
fi

cat > "$tmp/scripts/release-alpha-gate.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
scripts/check-milestone-closure.sh --fail-on-pending
SH
chmod +x "$tmp/scripts/release-alpha-gate.sh"

cat > "$tmp/.github/workflows/alpha-release-gate.yml" <<'YAML'
name: Alpha Release Gate
on:
  workflow_dispatch:
  push:
    branches:
      - main
jobs:
  alpha-release-gate:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run alpha release gate script
        run: scripts/release-alpha-gate.sh
      - name: Verify release promotion inputs
        run: scripts/verify-release-promotion-inputs.sh build/release-alpha-gate/release-summary.txt
      - name: Generate release publish manifest
        run: scripts/generate-release-publish-manifest.sh
      - name: Verify release publish manifest
        run: scripts/verify-release-publish-manifest.sh build/release-alpha-gate/release-publish-manifest.json
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when alpha-release workflow misses artifact upload contract" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/alpha-release-gate.yml" <<'YAML'
name: Alpha Release Gate
on:
  workflow_dispatch:
  push:
    branches:
      - main
jobs:
  alpha-release-gate:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run alpha release gate script
        run: scripts/release-alpha-gate.sh
      - name: Verify release promotion inputs
        run: scripts/verify-release-promotion-inputs.sh build/release-alpha-gate/release-summary.txt
      - name: Generate release publish manifest
        run: scripts/generate-release-publish-manifest.sh
      - name: Verify release publish manifest
        run: scripts/verify-release-publish-manifest.sh build/release-alpha-gate/release-publish-manifest.json
      - name: Upload alpha release gate artifacts
        uses: actions/upload-artifact@v4
        with:
          name: alpha-release-gate-artifacts
          path: build/release-alpha-gate
YAML

cat > "$tmp/.github/workflows/release-contract-smoke.yml" <<'YAML'
name: Release Contract Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  release-contract-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Verify alpha release gate workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Verify alpha release gate workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
      - name: Verify release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when release-contract-smoke workflow misses publish verifier coverage" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/release-contract-smoke.yml" <<'YAML'
name: Release Contract Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  release-contract-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Verify alpha release gate workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Verify alpha release gate workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Verify release promotion input verifier
        run: scripts/test-verify-release-promotion-inputs.sh
      - name: Verify release publish manifest generation
        run: scripts/test-generate-release-publish-manifest.sh
      - name: Verify release publish manifest verifier
        run: scripts/test-verify-release-publish-manifest.sh
      - name: Verify release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
YAML

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate replay capture contract
        run: scripts/test-replay-capture-contract.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
      - name: Validate zed grammar pin guard behavior
        run: scripts/test-zed-grammar-pin-guard.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses alpha-release guard test step" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
      - name: Validate zed grammar pin guard behavior
        run: scripts/test-zed-grammar-pin-guard.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses sec4-explain coverage contract test" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
      - name: Validate zed grammar pin guard behavior
        run: scripts/test-zed-grammar-pin-guard.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses replay-capture contract test" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate replay capture contract
        run: scripts/test-replay-capture-contract.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
      - name: Validate zed grammar pin guard behavior
        run: scripts/test-zed-grammar-pin-guard.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses replay-capture compatibility test" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate replay capture contract
        run: scripts/test-replay-capture-contract.sh
      - name: Validate replay capture compatibility
        run: scripts/test-replay-capture-compat.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
      - name: Validate zed grammar pin guard behavior
        run: scripts/test-zed-grammar-pin-guard.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses replay-stub-registry contract test" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate replay capture contract
        run: scripts/test-replay-capture-contract.sh
      - name: Validate replay capture compatibility
        run: scripts/test-replay-capture-compat.sh
      - name: Validate replay stub registry contract
        run: scripts/test-replay-stub-registry-contract.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
      - name: Validate zed grammar pin guard behavior
        run: scripts/test-zed-grammar-pin-guard.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses replay-cli-json contract tests" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate replay capture contract
        run: scripts/test-replay-capture-contract.sh
      - name: Validate replay capture compatibility
        run: scripts/test-replay-capture-compat.sh
      - name: Validate replay stub registry contract
        run: scripts/test-replay-stub-registry-contract.sh
      - name: Validate replay CLI json contract
        run: scripts/test-replay-cli-json-contract.sh
      - name: Validate replay CLI json contract guard behavior
        run: scripts/test-replay-cli-json-contract-guard.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
      - name: Validate zed grammar pin guard behavior
        run: scripts/test-zed-grammar-pin-guard.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses cross-impl guard test step" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate replay capture contract
        run: scripts/test-replay-capture-contract.sh
      - name: Validate replay capture compatibility
        run: scripts/test-replay-capture-compat.sh
      - name: Validate replay stub registry contract
        run: scripts/test-replay-stub-registry-contract.sh
      - name: Validate replay CLI json contract
        run: scripts/test-replay-cli-json-contract.sh
      - name: Validate replay CLI json contract guard behavior
        run: scripts/test-replay-cli-json-contract-guard.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
      - name: Validate zed grammar pin guard behavior
        run: scripts/test-zed-grammar-pin-guard.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses benchmark-trend guard test step" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate replay capture contract
        run: scripts/test-replay-capture-contract.sh
      - name: Validate replay capture compatibility
        run: scripts/test-replay-capture-compat.sh
      - name: Validate replay stub registry contract
        run: scripts/test-replay-stub-registry-contract.sh
      - name: Validate replay CLI json contract
        run: scripts/test-replay-cli-json-contract.sh
      - name: Validate replay CLI json contract guard behavior
        run: scripts/test-replay-cli-json-contract-guard.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses sec4-cli guard test step" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate replay capture contract
        run: scripts/test-replay-capture-contract.sh
      - name: Validate replay capture compatibility
        run: scripts/test-replay-capture-compat.sh
      - name: Validate replay stub registry contract
        run: scripts/test-replay-stub-registry-contract.sh
      - name: Validate replay CLI json contract
        run: scripts/test-replay-cli-json-contract.sh
      - name: Validate replay CLI json contract guard behavior
        run: scripts/test-replay-cli-json-contract-guard.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when naming-lock workflow misses zed-grammar-pin guard test step" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/naming-lock.yml" <<'YAML'
name: Naming Lock
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  naming-lock:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Validate sec4 explain audit coverage
        run: scripts/test-check-sec4-explain-audit-coverage.sh
      - name: Validate replay capture contract
        run: scripts/test-replay-capture-contract.sh
      - name: Validate replay capture compatibility
        run: scripts/test-replay-capture-compat.sh
      - name: Validate replay stub registry contract
        run: scripts/test-replay-stub-registry-contract.sh
      - name: Validate replay CLI json contract
        run: scripts/test-replay-cli-json-contract.sh
      - name: Validate replay CLI json contract guard behavior
        run: scripts/test-replay-cli-json-contract-guard.sh
      - name: Validate alpha release workflow contract
        run: scripts/test-alpha-release-workflow-contract.sh
      - name: Validate alpha release workflow guard behavior
        run: scripts/test-alpha-release-workflow-contract-guard.sh
      - name: Validate release-contract-smoke workflow contract
        run: scripts/test-release-contract-smoke-workflow-contract.sh
      - name: Validate release-contract-smoke workflow guard behavior
        run: scripts/test-release-contract-smoke-workflow-contract-guard.sh
      - name: Validate benchmark cross-impl workflow contract
        run: scripts/test-benchmark-cross-impl-workflow-contract.sh
      - name: Validate benchmark cross-impl workflow guard behavior
        run: scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
      - name: Validate benchmark trend workflow contract
        run: scripts/test-benchmark-trend-workflow-contract.sh
      - name: Validate benchmark trend workflow guard behavior
        run: scripts/test-benchmark-trend-workflow-contract-guard.sh
      - name: Validate sec4 CLI command contract
        run: scripts/test-sec4-cli-command-contract.sh
      - name: Validate sec4 CLI command contract guard behavior
        run: scripts/test-sec4-cli-command-contract-guard.sh
      - name: Validate sec4 run runtime-flag contract
        run: scripts/test-sec4-run-runtime-flag-contract.sh
      - name: Validate sec4 run runtime-flag contract guard behavior
        run: scripts/test-sec4-run-runtime-flag-contract-guard.sh
      - name: Validate zed grammar pin contract
        run: scripts/test-zed-grammar-pin.sh
      - name: Validate zed grammar pin guard behavior
        run: scripts/test-zed-grammar-pin-guard.sh
YAML

cat > "$tmp/.github/workflows/benchmark-smoke.yml" <<'YAML'
name: Benchmark Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  benchmark-scripts:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run benchmark script smoke tests
        run: |
          scripts/test-benchmark-smoke-closure-gate.sh
          scripts/test-benchmark-smoke-closure-gate-guard.sh
          scripts/test-benchmark-cross-impl-workflow-contract.sh
          scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
          scripts/test-benchmark-trend-workflow-contract.sh
          scripts/test-benchmark-trend-workflow-contract-guard.sh
          scripts/check-milestone-closure.sh --fail-on-pending
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when benchmark-smoke workflow misses closure fixture test coverage" >&2
  exit 1
fi

cat > "$tmp/.github/workflows/benchmark-smoke.yml" <<'YAML'
name: Benchmark Smoke
on:
  pull_request:
  push:
    branches:
      - main
jobs:
  benchmark-scripts:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Run benchmark script smoke tests
        run: |
          scripts/test-benchmark-smoke-closure-gate.sh
          scripts/test-benchmark-smoke-closure-gate-guard.sh
          scripts/test-benchmark-cross-impl-workflow-contract.sh
          scripts/test-benchmark-cross-impl-workflow-contract-guard.sh
          scripts/test-benchmark-trend-workflow-contract.sh
          scripts/test-benchmark-trend-workflow-contract-guard.sh
          scripts/test-check-milestone-closure.sh
          scripts/check-milestone-closure.sh --fail-on-pending
YAML

cat > "$tmp/.github/workflows/benchmark-cross-impl-evidence.yml" <<'YAML'
name: Benchmark Cross-Impl Evidence
on:
  workflow_dispatch:
jobs:
  cross-impl-benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Run cross-impl benchmark matrix (ping+decode)
        run: |
          benchmark-suite/scripts/run_comparison_matrix.sh \
            --impls sec4,node,go,rust \
            --endpoints ping,decode
      - name: Enforce benchmark evidence quality
        run: |
          scripts/check-benchmark-evidence-quality.sh \
            --matrix benchmark-suite/results/summaries/compare-matrix.json
      - name: Upload cross-impl benchmark evidence
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-cross-impl-evidence
          path: benchmark-suite/results
YAML

if "$root_dir/check-milestone-closure.sh" --repo-root "$tmp" --fail-on-pending >/dev/null 2>&1; then
  echo "expected pending failure when cross-impl workflow omits strict quality flag" >&2
  exit 1
fi

echo "check-milestone-closure test passed"
