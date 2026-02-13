#!/usr/bin/env bash
set -euo pipefail

out="${1:-benchmark-suite/results/env.json}"
mkdir -p "$(dirname "$out")"

cpu="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown)"
ram="$(sysctl -n hw.memsize 2>/dev/null || echo 0)"
os="$(uname -a)"
clang_v="$(clang --version 2>/dev/null | head -n1 || echo missing)"
rust_v="$(rustc --version 2>/dev/null || echo missing)"
go_v="$(go version 2>/dev/null || echo missing)"
node_v="$(node --version 2>/dev/null || echo missing)"

cat > "$out" <<JSON
{
  "cpu": "${cpu}",
  "ramBytes": "${ram}",
  "os": "${os}",
  "clang": "${clang_v}",
  "rust": "${rust_v}",
  "go": "${go_v}",
  "node": "${node_v}"
}
JSON

echo "wrote $out"
