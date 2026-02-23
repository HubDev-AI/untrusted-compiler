#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

out="$("${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" \
  --dry-run \
  --profile ping \
  --project-path examples/lasm-alpha-full \
  --request-path /health \
  --request-header 'Authorization: Bearer token123' \
  --duration 7s \
  --threads 2 \
  --connections 16 \
  --target-requests 4567 \
  --port 19094 \
  --instances 3 \
  --autoscale-max-instances 5 \
  --autoscale-target-connections 111 \
  --autoscale-check-ms 250 \
  --autoscale-scale-up-cooldown-ms 123 \
  --autoscale-scale-down-cooldown-ms 456 \
  --autoscale-scale-up-step 5 \
  --autoscale-scale-down-step 2 \
  --autoscale-saturation-boost-step 6 \
  --cluster-relay-workers 9 \
  --cluster-relay-queue 999 \
  --cluster-accept-workers 4 \
  --cluster-relay-accept-batch-max 321 \
  --cluster-relay-pump-batch-max 654 \
  --db-query-one-row-max-bytes 1001 \
  --db-query-one-row-max-columns 1002 \
  --db-sql-template-max-bytes 1003 \
  --db-params-max-bytes 1004 \
  --db-params-max-entries 1005 \
  --build-profile debug \
  --samples 3 \
  --wrk-processes 4 \
  --proxy-out results/summaries/custom-lasm-mode-compare-proxy.json \
  --fixed-out results/summaries/custom-lasm-mode-compare-fixed.json \
  --out results/summaries/custom-lasm-mode-compare.json \
  2>&1)"

if ! grep -q 'sec4 LASM cluster mode compare plan:' <<<"$out"; then
  echo "mode compare dry-run missing plan header" >&2
  exit 1
fi
if ! grep -q 'profile=ping' <<<"$out"; then
  echo "mode compare dry-run missing profile marker" >&2
  exit 1
fi
if ! grep -q "proxyOut=${root_dir}/results/summaries/custom-lasm-mode-compare-proxy.json" <<<"$out"; then
  echo "mode compare dry-run missing resolved proxy output path" >&2
  exit 1
fi
if ! grep -q "fixedOut=${root_dir}/results/summaries/custom-lasm-mode-compare-fixed.json" <<<"$out"; then
  echo "mode compare dry-run missing resolved fixed output path" >&2
  exit 1
fi
if ! grep -q "out=${root_dir}/results/summaries/custom-lasm-mode-compare.json" <<<"$out"; then
  echo "mode compare dry-run missing resolved comparison output path" >&2
  exit 1
fi
if ! grep -q 'clusterRelayWorkers=9' <<<"$out"; then
  echo "mode compare dry-run missing proxy relay workers passthrough" >&2
  exit 1
fi
if ! grep -q 'dbQueryOneRowMaxBytes=1001' <<<"$out"; then
  echo "mode compare dry-run missing db-query-one-row-max-bytes passthrough" >&2
  exit 1
fi
if ! grep -q 'dbParamsMaxEntries=1005' <<<"$out"; then
  echo "mode compare dry-run missing db-params-max-entries passthrough" >&2
  exit 1
fi
if ! grep -q 'buildProfile=debug' <<<"$out"; then
  echo "mode compare dry-run missing build profile marker" >&2
  exit 1
fi
if ! grep -q 'samples=3' <<<"$out"; then
  echo "mode compare dry-run missing samples marker" >&2
  exit 1
fi
if ! grep -q 'wrkProcesses=4' <<<"$out"; then
  echo "mode compare dry-run missing wrk-processes marker" >&2
  exit 1
fi
if ! grep -q 'fixedReusePortMode=true' <<<"$out"; then
  echo "mode compare dry-run missing delegated fixed reuse-port mode marker" >&2
  exit 1
fi
out_db_query_profile="$("${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" \
  --dry-run \
  --profile db-hot-query-one \
  --port 19095 \
  2>&1)"
if ! grep -q 'profile=db-hot-query-one' <<<"$out_db_query_profile"; then
  echo "mode compare db query-one dry-run missing profile marker" >&2
  exit 1
fi
if ! grep -q 'projectPath=benchmark-suite/services/sec4-lasm' <<<"$out_db_query_profile"; then
  echo "mode compare db query-one dry-run missing default project path" >&2
  exit 1
fi
if ! grep -q 'requestPath=/db/hot-query-one' <<<"$out_db_query_profile"; then
  echo "mode compare db query-one dry-run missing default request path" >&2
  exit 1
fi
if ! grep -q 'warmupPath=/db/hot-write' <<<"$out_db_query_profile"; then
  echo "mode compare db query-one dry-run missing default warmup path" >&2
  exit 1
fi
out_db_postgres_query_profile="$("${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" \
  --dry-run \
  --profile db-hot-postgres-query-one \
  --db-postgres-dsn-file /tmp/sec4-test-postgres.dsn \
  --port 19096 \
  2>&1)"
if ! grep -q 'profile=db-hot-postgres-query-one' <<<"$out_db_postgres_query_profile"; then
  echo "mode compare db postgres query-one dry-run missing profile marker" >&2
  exit 1
fi
if ! grep -q 'requestPath=/db/hot-query-one' <<<"$out_db_postgres_query_profile"; then
  echo "mode compare db postgres query-one dry-run missing default request path" >&2
  exit 1
fi
if ! grep -q 'dbPostgresDsnFile=/tmp/sec4-test-postgres.dsn' <<<"$out_db_postgres_query_profile"; then
  echo "mode compare db postgres query-one dry-run missing dsn passthrough marker" >&2
  exit 1
fi
if env -u SEC4_RT_LASM_DB_POSTGRES_DSN "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" \
  --dry-run \
  --profile db-hot-postgres-query-one \
  --port 19097 >/tmp/lasm-mode-compare-missing-postgres-dsn.log 2>&1; then
  echo "mode compare accepted db-hot-postgres-query-one profile without DSN" >&2
  exit 1
fi
if ! grep -q "postgres adapter requires --db-postgres-dsn-file or SEC4_RT_LASM_DB_POSTGRES_DSN" /tmp/lasm-mode-compare-missing-postgres-dsn.log; then
  echo "mode compare missing postgres profile DSN diagnostic" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --request-header invalid >/tmp/lasm-mode-compare-invalid-header.log 2>&1; then
  echo "mode compare accepted invalid request header" >&2
  exit 1
fi
if ! grep -q "request-header must include ':'" /tmp/lasm-mode-compare-invalid-header.log; then
  echo "mode compare missing invalid request-header diagnostic" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --cluster-relay-pump-batch-max nope >/tmp/lasm-mode-compare-invalid-pump.log 2>&1; then
  echo "mode compare accepted invalid relay pump batch value" >&2
  exit 1
fi
if ! grep -q "cluster-relay-pump-batch-max must be numeric" /tmp/lasm-mode-compare-invalid-pump.log; then
  echo "mode compare missing invalid relay pump batch diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --db-query-one-row-max-bytes nope >/tmp/lasm-mode-compare-invalid-db-query-row-bytes.log 2>&1; then
  echo "mode compare accepted invalid db-query-one-row-max-bytes value" >&2
  exit 1
fi
if ! grep -q "db-query-one-row-max-bytes must be numeric" /tmp/lasm-mode-compare-invalid-db-query-row-bytes.log; then
  echo "mode compare missing invalid db-query-one-row-max-bytes diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --samples 0 >/tmp/lasm-mode-compare-invalid-samples.log 2>&1; then
  echo "mode compare accepted invalid samples value" >&2
  exit 1
fi
if ! grep -q "samples must be >= 1" /tmp/lasm-mode-compare-invalid-samples.log; then
  echo "mode compare missing invalid samples diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --wrk-processes 0 >/tmp/lasm-mode-compare-invalid-wrk-processes.log 2>&1; then
  echo "mode compare accepted invalid wrk-processes value" >&2
  exit 1
fi
if ! grep -q "wrk-processes must be >= 1" /tmp/lasm-mode-compare-invalid-wrk-processes.log; then
  echo "mode compare missing invalid wrk-processes diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --wrk-processes nope >/tmp/lasm-mode-compare-invalid-wrk-processes-type.log 2>&1; then
  echo "mode compare accepted non-integer wrk-processes value" >&2
  exit 1
fi
if ! grep -q "wrk-processes must be an integer >= 1" /tmp/lasm-mode-compare-invalid-wrk-processes-type.log; then
  echo "mode compare missing invalid wrk-processes type diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --build-profile fast >/tmp/lasm-mode-compare-invalid-profile.log 2>&1; then
  echo "mode compare accepted invalid build profile value" >&2
  exit 1
fi
if ! grep -q "build-profile must be one of: debug, release" /tmp/lasm-mode-compare-invalid-profile.log; then
  echo "mode compare missing invalid build profile diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_mode_compare.sh" --dry-run --profile nope >/tmp/lasm-mode-compare-invalid-run-profile.log 2>&1; then
  echo "mode compare accepted invalid profile value" >&2
  exit 1
fi
if ! grep -q "profile must be one of: ping, db-hot-write, db-hot-write-tx, db-hot-query-one, db-hot-postgres-query-one" /tmp/lasm-mode-compare-invalid-run-profile.log; then
  echo "mode compare missing invalid profile diagnostic" >&2
  exit 1
fi

echo "run_lasm_cluster_mode_compare test passed"
