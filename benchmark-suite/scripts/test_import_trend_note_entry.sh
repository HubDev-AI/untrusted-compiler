#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

chapter="${tmp}/trend-note.md"
entry="${tmp}/entry.md"

cat > "$chapter" <<'EOF'
# Trend Note

Base content.
EOF

cat > "$entry" <<'EOF'
## Trend Entry (2026-02-13)

| Endpoint | Leader | p99 (ms) | Coverage (%) | Absolute Guard | Baseline Guard |
| --- | --- | ---: | ---: | --- | --- |
| ping | node | 18.20 | 90.00 | pass | pass |
EOF

"$root_dir/import_trend_note_entry.sh" --entry "$entry" --chapter "$chapter" >/dev/null

if ! grep -q '^## Trend Entry (2026-02-13)$' "$chapter"; then
  echo "entry was not imported" >&2
  exit 1
fi

"$root_dir/import_trend_note_entry.sh" --entry "$entry" --chapter "$chapter" >/dev/null

count="$(grep -c '^## Trend Entry (2026-02-13)$' "$chapter")"
if [ "$count" -ne 1 ]; then
  echo "entry import should be idempotent" >&2
  exit 1
fi

echo "import_trend_note_entry test passed"
