#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

out="$("${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" \
  --dry-run \
  --project-path examples/lasm-alpha-full \
  --request-path /health \
  --request-header 'Authorization: Bearer token123' \
  --duration 7s \
  --threads 2 \
  --connections 16 \
  --target-requests 4567 \
  --port 19091 \
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
  --db-base /tmp/lasm-db-probe \
  --db-adapter sqlite \
  --db-postgres-dsn-file /tmp/lasm-postgres-dsn.txt \
  --db-postgres-tls-mode require \
  --db-max-tx-handles 22 \
  --db-records-max 333 \
  --db-postgres-statement-cache-max 444 \
  --db-postgres-placeholder-cache-max 555 \
  --db-postgres-statement-timeout-ms 666 \
  --db-postgres-lock-timeout-ms 777 \
  --db-postgres-connect-timeout-ms 888 \
  --db-sqlite-busy-timeout-ms 999 \
  --db-sqlite-journal-mode WAL \
  --db-sqlite-synchronous NORMAL \
  --db-postgres-retryable-conflict-retry-max 10 \
  --db-sqlite-lock-retry-max 11 \
  --db-sqlite-lock-retry-delay-ms 12 \
  --build-profile debug \
  --samples 3 \
  --wrk-processes 4 \
  --out results/summaries/custom-lasm-capacity.json \
  2>&1)"

if ! grep -q 'projectPath=' <<<"$out"; then
  echo "lasm capacity probe dry-run missing project path output" >&2
  exit 1
fi
if ! grep -q 'requestPath=/health' <<<"$out"; then
  echo "lasm capacity probe dry-run missing requestPath output" >&2
  exit 1
fi
if ! grep -q 'profile=ping' <<<"$out"; then
  echo "lasm capacity probe dry-run missing default profile output" >&2
  exit 1
fi
if ! grep -q 'targetRequests=4567' <<<"$out"; then
  echo "lasm capacity probe dry-run missing targetRequests output" >&2
  exit 1
fi
if ! grep -q 'autoscaleScaleUpCooldownMs=123' <<<"$out"; then
  echo "lasm capacity probe dry-run missing scale-up cooldown output" >&2
  exit 1
fi
if ! grep -q 'autoscaleScaleDownCooldownMs=456' <<<"$out"; then
  echo "lasm capacity probe dry-run missing scale-down cooldown output" >&2
  exit 1
fi
if ! grep -q 'autoscaleScaleUpStep=5' <<<"$out"; then
  echo "lasm capacity probe dry-run missing scale-up step output" >&2
  exit 1
fi
if ! grep -q 'autoscaleScaleDownStep=2' <<<"$out"; then
  echo "lasm capacity probe dry-run missing scale-down step output" >&2
  exit 1
fi
if ! grep -q 'autoscaleSaturationBoostStep=6' <<<"$out"; then
  echo "lasm capacity probe dry-run missing saturation boost step output" >&2
  exit 1
fi
if ! grep -q 'clusterRelayWorkers=9' <<<"$out"; then
  echo "lasm capacity probe dry-run missing relay workers output" >&2
  exit 1
fi
if ! grep -q 'clusterRelayQueue=999' <<<"$out"; then
  echo "lasm capacity probe dry-run missing relay queue output" >&2
  exit 1
fi
if ! grep -q 'clusterAcceptWorkers=4' <<<"$out"; then
  echo "lasm capacity probe dry-run missing accept workers output" >&2
  exit 1
fi
if ! grep -q 'clusterRelayAcceptBatchMax=321' <<<"$out"; then
  echo "lasm capacity probe dry-run missing relay accept batch output" >&2
  exit 1
fi
if ! grep -q 'clusterRelayPumpBatchMax=654' <<<"$out"; then
  echo "lasm capacity probe dry-run missing relay pump batch output" >&2
  exit 1
fi
if ! grep -q 'dbBase=/tmp/lasm-db-probe' <<<"$out"; then
  echo "lasm capacity probe dry-run missing db-base output" >&2
  exit 1
fi
if ! grep -q 'dbAdapter=sqlite' <<<"$out"; then
  echo "lasm capacity probe dry-run missing db-adapter output" >&2
  exit 1
fi
if ! grep -q 'dbPostgresTlsMode=require' <<<"$out"; then
  echo "lasm capacity probe dry-run missing db-postgres-tls-mode output" >&2
  exit 1
fi
if ! grep -q 'dbMaxTxHandles=22' <<<"$out"; then
  echo "lasm capacity probe dry-run missing db-max-tx-handles output" >&2
  exit 1
fi
if ! grep -q 'dbSqliteJournalMode=WAL' <<<"$out"; then
  echo "lasm capacity probe dry-run missing db-sqlite-journal-mode output" >&2
  exit 1
fi
if ! grep -q 'dbSqliteLockRetryDelayMs=12' <<<"$out"; then
  echo "lasm capacity probe dry-run missing db-sqlite-lock-retry-delay-ms output" >&2
  exit 1
fi
if ! grep -q 'buildProfile=debug' <<<"$out"; then
  echo "lasm capacity probe dry-run missing build profile output" >&2
  exit 1
fi
if ! grep -q 'samples=3' <<<"$out"; then
  echo "lasm capacity probe dry-run missing samples output" >&2
  exit 1
fi
if ! grep -q 'wrkProcesses=4' <<<"$out"; then
  echo "lasm capacity probe dry-run missing wrkProcesses output" >&2
  exit 1
fi
if ! grep -q "out=${root_dir}/results/summaries/custom-lasm-capacity.json" <<<"$out"; then
  echo "lasm capacity probe dry-run missing resolved output path" >&2
  exit 1
fi
if ! grep -q "clusterStatusJson=${root_dir}/results/raw/sec4-lasm-cluster-capacity-status-19091.json" <<<"$out"; then
  echo "lasm capacity probe dry-run missing resolved cluster status json path" >&2
  exit 1
fi
if ! grep -q "keepClusterStatusJson=false" <<<"$out"; then
  echo "lasm capacity probe dry-run missing keep-cluster-status-json default marker" >&2
  exit 1
fi

out_keep_status="$("${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" \
  --dry-run \
  --keep-cluster-status-json \
  --port 19092 \
  2>&1)"
if ! grep -q "clusterStatusJson=${root_dir}/results/raw/sec4-lasm-cluster-capacity-status-19092.json" <<<"$out_keep_status"; then
  echo "lasm capacity probe dry-run missing keep-mode status json path" >&2
  exit 1
fi
if ! grep -q "keepClusterStatusJson=true" <<<"$out_keep_status"; then
  echo "lasm capacity probe dry-run missing keep-cluster-status-json enable marker" >&2
  exit 1
fi

out_db_profile="$("${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" \
  --dry-run \
  --profile db-hot-write \
  --port 19094 \
  2>&1)"
if ! grep -q 'profile=db-hot-write' <<<"$out_db_profile"; then
  echo "lasm capacity probe db profile dry-run missing profile marker" >&2
  exit 1
fi
if ! grep -q 'projectPath=.*/benchmark-suite/services/sec4-lasm' <<<"$out_db_profile"; then
  echo "lasm capacity probe db profile dry-run missing default db project path" >&2
  exit 1
fi
if ! grep -q 'requestPath=/db/hot-write' <<<"$out_db_profile"; then
  echo "lasm capacity probe db profile dry-run missing default db request path" >&2
  exit 1
fi
if ! grep -q 'dbAdapter=records-log' <<<"$out_db_profile"; then
  echo "lasm capacity probe db profile dry-run missing default db adapter override" >&2
  exit 1
fi

out_fixed="$("${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" \
  --dry-run \
  --fixed-reuse-port-mode \
  --instances 3 \
  --autoscale-max-instances 9 \
  --port 19093 \
  2>&1)"
if ! grep -q "fixedReusePortMode=true" <<<"$out_fixed"; then
  echo "lasm capacity probe dry-run missing fixed reuse-port mode marker" >&2
  exit 1
fi
if ! grep -q "autoscaleMaxInstances=3" <<<"$out_fixed"; then
  echo "lasm capacity probe fixed reuse-port mode should force autoscale max to instances" >&2
  exit 1
fi
if ! grep -q "clusterStatusJson=n/a (fixed-reuse-port-mode)" <<<"$out_fixed"; then
  echo "lasm capacity probe fixed reuse-port mode should suppress cluster status artifact path" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --fixed-reuse-port-mode --cluster-relay-workers 2 >/tmp/lasm-capacity-probe-fixed-invalid-relay-workers.log 2>&1; then
  echo "lasm capacity probe accepted relay workers override in fixed reuse-port mode" >&2
  exit 1
fi
if ! grep -q "cluster-relay-workers is not supported in fixed-reuse-port-mode" /tmp/lasm-capacity-probe-fixed-invalid-relay-workers.log; then
  echo "lasm capacity probe missing fixed-mode relay-workers diagnostic" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --fixed-reuse-port-mode --keep-cluster-status-json >/tmp/lasm-capacity-probe-fixed-invalid-status.log 2>&1; then
  echo "lasm capacity probe accepted keep-cluster-status-json in fixed reuse-port mode" >&2
  exit 1
fi
if ! grep -q "keep-cluster-status-json is not supported in fixed-reuse-port-mode" /tmp/lasm-capacity-probe-fixed-invalid-status.log; then
  echo "lasm capacity probe missing fixed-mode keep-status diagnostic" >&2
  exit 1
fi

if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --request-header invalid >/tmp/lasm-capacity-probe-invalid.log 2>&1; then
  echo "lasm capacity probe accepted invalid request header" >&2
  exit 1
fi
if ! grep -q "request-header must include ':'" /tmp/lasm-capacity-probe-invalid.log; then
  echo "lasm capacity probe invalid request-header error missing" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --cluster-relay-pump-batch-max nope >/tmp/lasm-capacity-probe-invalid-pump-batch.log 2>&1; then
  echo "lasm capacity probe accepted invalid relay pump batch value" >&2
  exit 1
fi
if ! grep -q "cluster-relay-pump-batch-max must be numeric" /tmp/lasm-capacity-probe-invalid-pump-batch.log; then
  echo "lasm capacity probe invalid relay pump batch error missing" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --db-max-tx-handles nope >/tmp/lasm-capacity-probe-invalid-db-max-tx.log 2>&1; then
  echo "lasm capacity probe accepted invalid db-max-tx-handles value" >&2
  exit 1
fi
if ! grep -q "db-max-tx-handles must be numeric" /tmp/lasm-capacity-probe-invalid-db-max-tx.log; then
  echo "lasm capacity probe invalid db-max-tx-handles error missing" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --samples 0 >/tmp/lasm-capacity-probe-invalid-samples.log 2>&1; then
  echo "lasm capacity probe accepted invalid samples value" >&2
  exit 1
fi
if ! grep -q "samples must be >= 1" /tmp/lasm-capacity-probe-invalid-samples.log; then
  echo "lasm capacity probe invalid samples error missing" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --wrk-processes 0 >/tmp/lasm-capacity-probe-invalid-wrk-processes.log 2>&1; then
  echo "lasm capacity probe accepted invalid wrk-processes value" >&2
  exit 1
fi
if ! grep -q "wrk-processes must be >= 1" /tmp/lasm-capacity-probe-invalid-wrk-processes.log; then
  echo "lasm capacity probe invalid wrk-processes error missing" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --wrk-processes nope >/tmp/lasm-capacity-probe-invalid-wrk-processes-type.log 2>&1; then
  echo "lasm capacity probe accepted non-integer wrk-processes value" >&2
  exit 1
fi
if ! grep -q "wrk-processes must be an integer >= 1" /tmp/lasm-capacity-probe-invalid-wrk-processes-type.log; then
  echo "lasm capacity probe invalid wrk-processes type error missing" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --build-profile fast >/tmp/lasm-capacity-probe-invalid-profile.log 2>&1; then
  echo "lasm capacity probe accepted invalid build profile value" >&2
  exit 1
fi
if ! grep -q "build-profile must be one of: debug, release" /tmp/lasm-capacity-probe-invalid-profile.log; then
  echo "lasm capacity probe invalid build profile error missing" >&2
  exit 1
fi
if "${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh" --dry-run --profile nope >/tmp/lasm-capacity-probe-invalid-run-profile.log 2>&1; then
  echo "lasm capacity probe accepted invalid profile value" >&2
  exit 1
fi
if ! grep -q "profile must be one of: ping, db-hot-write, db-hot-write-tx" /tmp/lasm-capacity-probe-invalid-run-profile.log; then
  echo "lasm capacity probe invalid profile error missing" >&2
  exit 1
fi

echo "run_lasm_cluster_capacity_probe test passed"
