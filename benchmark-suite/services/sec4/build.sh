#!/usr/bin/env bash
set -euo pipefail

service_dir="$(cd "$(dirname "$0")" && pwd)"

mkdir -p "$service_dir/build"
cargo run -q -p sec4 -- build --path "$service_dir" --emit c \
  | awk 'started || /^#include / { started = 1; print }' \
  >"$service_dir/build/generated.c"

cp "$service_dir/runtime/benchmark_runtime.h" "$service_dir/build/sec4_runtime.h"
cp "$service_dir/runtime/benchmark_runtime.c" "$service_dir/build/sec4_runtime.c"

cc -O2 -std=c11 "$service_dir/build/generated.c" "$service_dir/build/sec4_runtime.c" -o "$service_dir/sec4-bench-server"

echo "built $service_dir/sec4-bench-server"
