#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checker="${root_dir}/scripts/check-m17-operator-handoff-readiness.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

repo_dir="${tmp_dir}/repo"
mkdir -p \
  "${repo_dir}/scripts" \
  "${repo_dir}/benchmark-suite/scripts" \
  "${repo_dir}/.github/workflows" \
  "${repo_dir}/docs/book"

cat > "${repo_dir}/scripts/smoke-sec4-run-hello-api.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
SH
chmod +x "${repo_dir}/scripts/smoke-sec4-run-hello-api.sh"

cat > "${repo_dir}/scripts/check-runtime-smoke-bundle.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
SH
chmod +x "${repo_dir}/scripts/check-runtime-smoke-bundle.sh"

cat > "${repo_dir}/scripts/check-milestone-closure.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
SH
chmod +x "${repo_dir}/scripts/check-milestone-closure.sh"

cat > "${repo_dir}/benchmark-suite/scripts/update_trend_note_from_ci.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
SH
chmod +x "${repo_dir}/benchmark-suite/scripts/update_trend_note_from_ci.sh"

cat > "${repo_dir}/scripts/check-zed-extension-operator-readiness.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
SH
chmod +x "${repo_dir}/scripts/check-zed-extension-operator-readiness.sh"

cat > "${repo_dir}/scripts/test-runtime-smoke-workflow-contract.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
SH
chmod +x "${repo_dir}/scripts/test-runtime-smoke-workflow-contract.sh"

cat > "${repo_dir}/scripts/test-runtime-smoke-workflow-contract-guard.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
SH
chmod +x "${repo_dir}/scripts/test-runtime-smoke-workflow-contract-guard.sh"

cat > "${repo_dir}/scripts/test-check-runtime-smoke-bundle.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
SH
chmod +x "${repo_dir}/scripts/test-check-runtime-smoke-bundle.sh"

cat > "${repo_dir}/scripts/run-naming-lock-contract-suite.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

scripts/test-runtime-smoke-workflow-contract.sh
scripts/test-runtime-smoke-workflow-contract-guard.sh
scripts/test-check-runtime-smoke-bundle.sh
scripts/test-check-m17-operator-handoff-readiness.sh
SH
chmod +x "${repo_dir}/scripts/run-naming-lock-contract-suite.sh"

cat > "${repo_dir}/.github/workflows/runtime-smoke.yml" <<'YAML'
name: Runtime Smoke

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  runtime-smoke:
    runs-on: ubuntu-latest
    steps:
      - name: Run sec4 hello-api operator smoke (default)
        run: scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke/default
      - name: Run sec4 hello-api operator smoke (max-body)
        run: scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir build/runtime-smoke/max-body
      - name: Validate runtime smoke bundle
        run: scripts/check-runtime-smoke-bundle.sh --artifacts-root build/runtime-smoke --index-path build/runtime-smoke/runtime-smoke-branch-index.json
YAML

cat > "${repo_dir}/.github/workflows/naming-lock.yml" <<'YAML'
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
      - name: Run naming-lock contract suite
        run: scripts/run-naming-lock-contract-suite.sh
YAML

cat > "${repo_dir}/docs/book/464-m17-operator-handoff-checklist-and-readiness-verifier.md" <<'MD'
# 464 M17 Operator Handoff Checklist and Readiness Verifier

1. Default runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>/default`
2. Max-body runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir <tmp>/max-body`
3. Bundle validation:
   - `scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json`
4. Closure readiness:
   - `scripts/check-milestone-closure.sh --fail-on-pending`
5. Zed extension operator readiness:
   - `scripts/check-zed-extension-operator-readiness.sh --stage-output <tmp>/zed-extension-release-operator`
6. Trend-note local fallback:
   - `benchmark-suite/scripts/update_trend_note_from_ci.sh --prefer-local`
MD

"${checker}" --repo-root "${repo_dir}" >/dev/null

cat > "${repo_dir}/docs/book/464-m17-operator-handoff-checklist-and-readiness-verifier.md" <<'MD'
# 464 M17 Operator Handoff Checklist and Readiness Verifier

1. Default runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>/default`
2. Max-body runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir <tmp>/max-body`
3. Bundle validation:
   - `scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json`
4. Closure readiness:
   - `scripts/check-milestone-closure.sh --fail-on-pending`
5. Trend-note local fallback:
   - `benchmark-suite/scripts/update_trend_note_from_ci.sh --prefer-local`
MD

if "${checker}" --repo-root "${repo_dir}" >"${tmp_dir}/missing-zed-token.log" 2>&1; then
  echo "expected M17 handoff checker to fail when checklist chapter misses zed readiness token" >&2
  exit 1
fi
if ! rg -Fq 'missing handoff chapter token: scripts/check-zed-extension-operator-readiness.sh --stage-output <tmp>/zed-extension-release-operator' "${tmp_dir}/missing-zed-token.log"; then
  echo "expected missing-token diagnostic for zed readiness command" >&2
  exit 1
fi

cat > "${repo_dir}/docs/book/464-m17-operator-handoff-checklist-and-readiness-verifier.md" <<'MD'
# 464 M17 Operator Handoff Checklist and Readiness Verifier

1. Default runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>/default`
2. Max-body runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir <tmp>/max-body`
3. Bundle validation:
   - `scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json`
4. Closure readiness:
   - `scripts/check-milestone-closure.sh --fail-on-pending`
5. Zed extension operator readiness:
   - `scripts/check-zed-extension-operator-readiness.sh --stage-output <tmp>/zed-extension-release-operator`
6. Trend-note local fallback:
   - `benchmark-suite/scripts/update_trend_note_from_ci.sh --prefer-local`
MD

chmod -x "${repo_dir}/benchmark-suite/scripts/update_trend_note_from_ci.sh"
if "${checker}" --repo-root "${repo_dir}" >"${tmp_dir}/non-executable.log" 2>&1; then
  echo "expected M17 handoff checker to fail when trend-note updater is not executable" >&2
  exit 1
fi
if ! rg -Fq 'trend-note updater is not executable' "${tmp_dir}/non-executable.log"; then
  echo "expected non-executable diagnostic for trend-note updater" >&2
  exit 1
fi

chmod +x "${repo_dir}/benchmark-suite/scripts/update_trend_note_from_ci.sh"
cat > "${repo_dir}/docs/book/464-m17-operator-handoff-checklist-and-readiness-verifier.md" <<'MD'
# 464 M17 Operator Handoff Checklist and Readiness Verifier

1. Default runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>/default`
2. Max-body runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir <tmp>/max-body`
3. Bundle validation:
   - `scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json`
4. Closure readiness:
   - `scripts/check-milestone-closure.sh --fail-on-pending`
5. Zed extension operator readiness:
   - `scripts/check-zed-extension-operator-readiness.sh --stage-output <tmp>/zed-extension-release-operator`
6. Trend-note local fallback:
   - `benchmark-suite/scripts/update_trend_note_from_ci.sh --prefer-local`
MD

"${checker}" --repo-root "${repo_dir}" >/dev/null

cat > "${repo_dir}/docs/book/464-m17-operator-handoff-checklist-and-readiness-verifier.md" <<'MD'
# 464 M17 Operator Handoff Checklist and Readiness Verifier

1. Default runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>/default`
2. Max-body runtime-smoke:
   - `scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir <tmp>/max-body`
3. Bundle validation:
   - `scripts/check-runtime-smoke-bundle.sh --artifacts-root <tmp> --index-path <tmp>/runtime-smoke-branch-index.json`
4. Closure readiness:
   - `scripts/check-milestone-closure.sh --fail-on-pending`
5. Zed extension operator readiness:
   - `scripts/check-zed-extension-operator-readiness.sh --stage-output <tmp>/zed-extension-release-operator`
6. Trend-note local fallback:
   - `benchmark-suite/scripts/update_trend_note_from_ci.sh --prefer-local`
MD

if ! "${checker}" --repo-root "${repo_dir}" >/dev/null; then
  echo "expected M17 handoff readiness checker to pass with all tokens and executable scripts" >&2
  exit 1
fi

echo "m17 operator handoff readiness checker test passed"
