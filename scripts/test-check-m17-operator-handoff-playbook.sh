#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checker_script="${root_dir}/scripts/check-m17-operator-handoff-playbook.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

repo_dir="${tmp_dir}/repo"
mkdir -p "${repo_dir}/scripts" "${repo_dir}/docs/book"

for script_name in \
  run-m17-operator-handoff-quickstart.sh \
  run-m17-operator-handoff-ci-smoke.sh \
  summarize-m17-operator-handoff-readiness.sh \
  inspect-m17-operator-handoff-artifacts.sh \
  build-m17-operator-release-packet.sh \
  check-milestone-closure.sh; do
  cat > "${repo_dir}/scripts/${script_name}" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
SCRIPT
  chmod +x "${repo_dir}/scripts/${script_name}"
done

cat > "${repo_dir}/docs/book/473-m17-operator-handoff-final-playbook.md" <<'MD'
# 473 M17 Operator Handoff Final Playbook

## Bundle A: Local Validation Flow

1. `scripts/check-m17-operator-handoff-readiness.sh --repo-root .`
2. `scripts/run-m17-operator-handoff-quickstart.sh --repo-root . --project examples/hello-api --artifacts-root build/runtime-smoke`
3. `scripts/summarize-m17-operator-handoff-readiness.sh --repo-root . --artifacts-root build/operator-handoff-smoke --format text`

## Bundle B: CI Smoke and Artifact Inspection Flow

1. `scripts/run-m17-operator-handoff-ci-smoke.sh --repo-root . --artifacts-root build/operator-handoff-smoke`
2. `scripts/inspect-m17-operator-handoff-artifacts.sh --repo-root . --artifacts-root build/operator-handoff-smoke --format text`
3. `scripts/check-milestone-closure.sh --repo-root . --fail-on-pending`

## Bundle C: Release Packet Assembly Flow

1. `scripts/build-m17-operator-release-packet.sh --repo-root . --artifacts-root build/operator-handoff-smoke --output-dir build/operator-release-packet`
2. `scripts/summarize-m17-operator-handoff-readiness.sh --repo-root . --artifacts-root build/operator-handoff-smoke --format json`
3. `scripts/check-milestone-closure.sh --repo-root . --format json --fail-on-pending`

Artifacts:
- `build/operator-release-packet/release-packet.json`
- `build/operator-release-packet/artifact-manifest.txt`
MD

"${checker_script}" --repo-root "${repo_dir}" >/dev/null

cat > "${repo_dir}/docs/book/473-m17-operator-handoff-final-playbook.md" <<'MD'
# 473 M17 Operator Handoff Final Playbook

## Bundle A: Local Validation Flow

1. `scripts/check-m17-operator-handoff-readiness.sh --repo-root .`
MD

if "${checker_script}" --repo-root "${repo_dir}" >"${tmp_dir}/missing-token.log" 2>&1; then
  echo "expected M17 playbook checker to fail when required bundle tokens are missing" >&2
  exit 1
fi
if ! rg -Fq 'missing playbook chapter token:' "${tmp_dir}/missing-token.log"; then
  echo "expected deterministic missing-token diagnostic from M17 playbook checker" >&2
  exit 1
fi

echo "m17 operator handoff playbook check test passed"
