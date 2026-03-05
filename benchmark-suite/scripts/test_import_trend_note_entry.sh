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

| Endpoint | Leader | p99 (ms) | Coverage (%) | RSS (KB) | Absolute Guard | Baseline Guard |
| --- | --- | ---: | ---: | ---: | --- | --- |
| ping | node | 18.20 | 90.00 | n/a | pass | pass |
EOF

entry_updated="${tmp}/entry-updated.md"
cat > "$entry_updated" <<'EOF'
## Trend Entry (2026-02-13)

| Endpoint | Leader | p99 (ms) | Coverage (%) | RSS (KB) | Absolute Guard | Baseline Guard |
| --- | --- | ---: | ---: | ---: | --- | --- |
| ping | go | 12.00 | 91.00 | n/a | pass | pass |
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

"$root_dir/import_trend_note_entry.sh" --entry "$entry_updated" --chapter "$chapter" --replace-existing >/dev/null

if ! grep -q '| ping | go | 12.00 | 91.00 | n/a | pass | pass |' "$chapter"; then
  echo "replace-existing did not refresh existing trend entry content" >&2
  exit 1
fi

count_after_replace="$(grep -c '^## Trend Entry (2026-02-13)$' "$chapter")"
if [ "$count_after_replace" -ne 1 ]; then
  echo "replace-existing should keep a single heading instance" >&2
  exit 1
fi

echo "import_trend_note_entry test passed"
