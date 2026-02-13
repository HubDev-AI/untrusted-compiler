#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$root_dir/check-sec4-explain-audit-coverage.sh" >/dev/null

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

audit_file="${tmp}/audit.rs"
explain_file="${tmp}/main.rs"

cat > "$audit_file" <<'EOF'
fn demo() {
  findings.push(finding("A_TEST_FINDING", AuditSeverity::LOW, "x", json!({}), "msg"));
  findings.push(finding("B_TEST_FINDING", AuditSeverity::LOW, "x", json!({}), "msg"));
}
EOF

cat > "$explain_file" <<'EOF'
match code {
  "A_TEST_FINDING" => {
    return ("A", "A", &POLICY_FIXES, "docs/book/x.md");
  }
  _ => {}
}
EOF

if "$root_dir/check-sec4-explain-audit-coverage.sh" --audit-file "$audit_file" --explain-file "$explain_file" >/dev/null 2>&1; then
  echo "expected missing mapping failure" >&2
  exit 1
fi

cat > "$explain_file" <<'EOF'
match code {
  "A_TEST_FINDING" => {
    return ("A", "A", &POLICY_FIXES, "docs/book/x.md");
  }
  "B_TEST_FINDING" => {
    return ("B", "B", &POLICY_FIXES, "docs/book/x.md");
  }
  _ => {}
}
EOF

"$root_dir/check-sec4-explain-audit-coverage.sh" --audit-file "$audit_file" --explain-file "$explain_file" >/dev/null

echo "check-sec4-explain-audit-coverage test passed"
