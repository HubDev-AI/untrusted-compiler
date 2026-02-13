#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

entry="${tmp}/trend-note-entry.md"
chapter="${tmp}/trend-note.md"

cat > "$entry" <<'EOF'
## Trend Entry (2026-02-13)

| Endpoint | Leader | p99 (ms) | Coverage (%) | Absolute Guard | Baseline Guard |
| --- | --- | ---: | ---: | --- | --- |
| ping | node | 18.20 | 90.00 | pass | pass |
EOF

cat > "$chapter" <<'EOF'
# Trend Note
EOF

dry_out="$("$root_dir/update_trend_note_from_ci.sh" --dry-run --repo HubDev-AI/untrusted-compiler --out-dir "$tmp" --chapter "$chapter")"
if ! grep -q 'fetch_trend_artifact.sh --repo HubDev-AI/untrusted-compiler --out-dir' <<<"$dry_out"; then
  echo "dry-run missing fetch command" >&2
  exit 1
fi
if ! grep -q 'import_trend_note_entry.sh --chapter' <<<"$dry_out"; then
  echo "dry-run missing import command" >&2
  exit 1
fi

"$root_dir/update_trend_note_from_ci.sh" --entry "$entry" --chapter "$chapter" >/dev/null

if ! grep -q '^## Trend Entry (2026-02-13)$' "$chapter"; then
  echo "entry not imported into chapter" >&2
  exit 1
fi

echo "update_trend_note_from_ci test passed"
