#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

out="$($root_dir/scripts/preflight.sh --impls sec4,node --dry-run-only)"
if ! grep -q '^OK       HTTP probing (curl)$' <<<"$out"; then
  echo "preflight missing curl check" >&2
  exit 1
fi
if ! grep -q '^OK       JSON processing (jq)$' <<<"$out"; then
  echo "preflight missing jq check" >&2
  exit 1
fi
if ! grep -q 'Untrusted<T> compiler runner' <<<"$out"; then
  echo "preflight missing sec4 tool check" >&2
  exit 1
fi

out_equals="$($root_dir/scripts/preflight.sh --impls=sec4,node --dry-run-only)"
if ! grep -q '^preflight passed$' <<<"$out_equals"; then
  echo "preflight did not pass with --impls= syntax" >&2
  exit 1
fi

out_lasm="$($root_dir/scripts/preflight.sh --impls sec4-lasm --dry-run-only)"
if ! grep -q 'Untrusted<T> compiler runner' <<<"$out_lasm"; then
  echo "preflight missing sec4-lasm tool check" >&2
  exit 1
fi
if ! grep -q '^preflight passed$' <<<"$out_lasm"; then
  echo "preflight sec4-lasm mode did not pass" >&2
  exit 1
fi

if "$root_dir/scripts/preflight.sh" --impls unknown --dry-run-only >/dev/null 2>&1; then
  echo "expected unknown implementation to fail" >&2
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
for cmd in curl jq wrk cargo cc node go; do
  cat > "${tmp}/${cmd}" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${tmp}/${cmd}"
done

fallback_out="$(PATH="${tmp}:/usr/bin:/bin:/usr/sbin:/sbin" "$root_dir/scripts/preflight.sh" --impls sec4,node,go,rust)"
if ! grep -q '^OK       load generator (wrk fallback) (wrk)$' <<<"$fallback_out"; then
  echo "preflight missing wrk fallback load-generator check" >&2
  exit 1
fi
if ! grep -q '^preflight passed$' <<<"$fallback_out"; then
  echo "preflight fallback mode did not pass" >&2
  exit 1
fi

if BENCH_REQUIRE_WRK2=1 PATH="${tmp}:/usr/bin:/bin:/usr/sbin:/sbin" \
  "$root_dir/scripts/preflight.sh" --impls sec4,node >/tmp/preflight-strict-wrk2.log 2>&1; then
  echo "preflight strict wrk2 mode unexpectedly passed without wrk2" >&2
  exit 1
fi
if ! grep -q '^MISSING  load generator (wrk2 required)$' /tmp/preflight-strict-wrk2.log; then
  echo "preflight strict wrk2 mode missing expected diagnostic" >&2
  exit 1
fi

echo "preflight test passed"
