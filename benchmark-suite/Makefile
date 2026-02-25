.PHONY: help db-up db-down db-schema env preflight preflight-dry bench-ping bench-decode bench-users bench-users-get bench-profile bench-step-profile analyze-step-profile compare-step-matrix bench-step-matrix bench-step-matrix-dry bench-matrix bench-matrix-dry bench-matrix-lasm-postgres bench-matrix-lasm-postgres-dry bench-full bench-full-dry bench-full-lasm-postgres bench-full-lasm-postgres-dry bench-alpha-postgres-suite bench-alpha-postgres-suite-dry bench-alpha-postgres-suite-local bench-alpha-postgres-suite-local-dry bench-full-saturation bench-full-saturation-dry bench-full-saturation-throughput bench-full-saturation-throughput-dry bench-full-saturation-latency bench-full-saturation-latency-dry bench-full-saturation-presets bench-full-saturation-presets-dry sec4-capacity-probe lasm-cluster-capacity-probe lasm-cluster-mode-compare lasm-cluster-saturation-boost-matrix lasm-cluster-saturation-boost-analyze lasm-cluster-saturation-boost-verify lasm-cluster-saturation-boost-summary lasm-cluster-saturation-boost-bundle artifact-manifest verify-bundle summarize report compare compare-matrix analyze-matrix publish-report test-scripts test-services

IMPL ?= sec4
IMPLS ?= sec4,sec4-lasm,node,go,rust
ENDPOINTS ?= ping,decode,users-post,users-get
BASE_URL ?= http://127.0.0.1:8080
ENDPOINT ?= ping
STEP_SUMMARY ?= results/summaries/$(IMPL)-$(ENDPOINT)-step.json
STEP_ANALYSIS ?= results/summaries/$(IMPL)-$(ENDPOINT)-step-analysis.json
STEP_MATRIX ?= results/summaries/step-matrix.json
ARTIFACT_MANIFEST ?= results/artifact-manifest.json
CAPACITY_ENDPOINT ?= ping
CAPACITY_TARGET_REQUESTS ?= 1000000
LASM_CAPACITY_PROJECT_PATH ?= examples/lasm-alpha-full
LASM_CAPACITY_TARGET_REQUESTS ?= 1000000
BENCH_LASM_DB_ADAPTER ?=
BENCH_LASM_DB_POSTGRES_DSN_FILE ?=
LASM_DB_MATRIX_IMPLS ?= sec4-lasm
LASM_DB_MATRIX_ENDPOINTS ?= db-hot-write,db-hot-write-tx,db-hot-query-one,db-records
BENCH_ALPHA_POSTGRES_RESET_BETWEEN_PHASES ?= false
BENCH_LOCAL_POSTGRES_KEEP_UP ?= false
BENCH_LOCAL_POSTGRES_RESET ?= false
LASM_CAPACITY_BOOST_STEPS ?= 2,4,6
LASM_CAPACITY_PROFILE ?= ping
LASM_CAPACITY_WARMUP_PATH ?=
LASM_CAPACITY_DURATION ?= 40s
LASM_CAPACITY_THREADS ?= 8
LASM_CAPACITY_CONNECTIONS ?= 256
LASM_CAPACITY_CLUSTER_RELAY_WORKERS ?=
LASM_CAPACITY_CLUSTER_RELAY_QUEUE ?=
LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS ?=
LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX ?=
LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX ?=
LASM_CAPACITY_FIXED_REUSE_PORT_MODE ?= false
LASM_SATURATION_MATRIX ?= results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json
LASM_SATURATION_ANALYSIS ?= results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json
LASM_SATURATION_VERIFY_OUT ?= results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json
LASM_SATURATION_SUMMARY ?= results/summaries/sec4-lasm-cluster-saturation-boost-summary.md
LASM_SATURATION_VERIFY_INPUT ?=
SATURATION_SUMMARY ?=
FULL_SATURATION_SKIP_VERIFY ?= false
FULL_LASM_INCLUDE_MODE_COMPARE ?= false
LASM_SATURATION_THROUGHPUT_IMPLS ?= sec4-lasm
LASM_SATURATION_THROUGHPUT_ENDPOINTS ?= ping
LASM_SATURATION_THROUGHPUT_BOOST_STEPS ?= 2,4,6,8
LASM_SATURATION_THROUGHPUT_PROFILE ?= ping
LASM_SATURATION_THROUGHPUT_WARMUP_PATH ?=
LASM_SATURATION_THROUGHPUT_DURATION ?= 60s
LASM_SATURATION_THROUGHPUT_THREADS ?= 12
LASM_SATURATION_THROUGHPUT_CONNECTIONS ?= 512
LASM_SATURATION_THROUGHPUT_TARGET_REQUESTS ?= 1000000
LASM_SATURATION_THROUGHPUT_RELAY_WORKERS ?= 32
LASM_SATURATION_THROUGHPUT_RELAY_QUEUE ?= 4096
LASM_SATURATION_THROUGHPUT_RELAY_PUMP_BATCH_MAX ?=
LASM_SATURATION_THROUGHPUT_FIXED_REUSE_PORT_MODE ?= false
LASM_SATURATION_LATENCY_IMPLS ?= sec4-lasm
LASM_SATURATION_LATENCY_ENDPOINTS ?= ping
LASM_SATURATION_LATENCY_BOOST_STEPS ?= 1,2,3
LASM_SATURATION_LATENCY_PROFILE ?= ping
LASM_SATURATION_LATENCY_WARMUP_PATH ?=
LASM_SATURATION_LATENCY_DURATION ?= 30s
LASM_SATURATION_LATENCY_THREADS ?= 4
LASM_SATURATION_LATENCY_CONNECTIONS ?= 64
LASM_SATURATION_LATENCY_TARGET_REQUESTS ?= 250000
LASM_SATURATION_LATENCY_RELAY_WORKERS ?= 16
LASM_SATURATION_LATENCY_RELAY_QUEUE ?= 2048
LASM_SATURATION_LATENCY_RELAY_PUMP_BATCH_MAX ?=
LASM_SATURATION_LATENCY_FIXED_REUSE_PORT_MODE ?= false

SATURATION_FIXED_REUSE_PORT_FLAG = $(if $(filter true,$(LASM_CAPACITY_FIXED_REUSE_PORT_MODE)),--saturation-fixed-reuse-port-mode,)
FIXED_REUSE_PORT_FLAG = $(if $(filter true,$(LASM_CAPACITY_FIXED_REUSE_PORT_MODE)),--fixed-reuse-port-mode,)
LASM_MODE_COMPARE_FLAG = $(if $(filter true,$(FULL_LASM_INCLUDE_MODE_COMPARE)),--include-lasm-mode-compare,)
SATURATION_WARMUP_PATH_FLAG = $(if $(strip $(LASM_CAPACITY_WARMUP_PATH)),--warmup-path "$(LASM_CAPACITY_WARMUP_PATH)",)
FULL_SATURATION_WARMUP_PATH_FLAG = $(if $(strip $(LASM_CAPACITY_WARMUP_PATH)),--saturation-warmup-path "$(LASM_CAPACITY_WARMUP_PATH)",)
LASM_DB_ADAPTER_FLAG = $(if $(strip $(BENCH_LASM_DB_ADAPTER)),--lasm-db-adapter "$(BENCH_LASM_DB_ADAPTER)",)
LASM_DB_POSTGRES_DSN_FILE_FLAG = $(if $(strip $(BENCH_LASM_DB_POSTGRES_DSN_FILE)),--lasm-db-postgres-dsn-file "$(BENCH_LASM_DB_POSTGRES_DSN_FILE)",)
ALPHA_POSTGRES_RESET_BETWEEN_PHASES_FLAG = $(if $(filter true,$(BENCH_ALPHA_POSTGRES_RESET_BETWEEN_PHASES)),--reset-db-between-phases,)
BENCH_LOCAL_POSTGRES_KEEP_UP_FLAG = $(if $(filter true,$(BENCH_LOCAL_POSTGRES_KEEP_UP)),--keep-up,)
BENCH_LOCAL_POSTGRES_RESET_FLAG = $(if $(filter true,$(BENCH_LOCAL_POSTGRES_RESET)),--reset-db,)

help:
	@echo "Targets:"
	@echo "  db-up        Start benchmark postgres"
	@echo "  db-down      Stop benchmark postgres"
	@echo "  db-schema    Apply benchmark schema"
	@echo "  env          Capture environment metadata"
	@echo "  preflight    Validate required benchmark tooling for IMPLS=$(IMPLS)"
	@echo "  preflight-dry Validate dry-run tooling only (skips wrk2 requirement)"
	@echo "  bench-ping   Run ping benchmark (requires service at $(BASE_URL))"
	@echo "  bench-decode Run decode benchmark (requires service at $(BASE_URL))"
	@echo "  bench-users  Run users write benchmark (requires service at $(BASE_URL))"
	@echo "  bench-users-get Run users read benchmark (requires service at $(BASE_URL))"
	@echo "  bench-profile Run one named profile via script (ENDPOINT=ping|decode|users-post|users-get|db-hot-write|db-hot-write-tx|db-hot-query-one|db-records)"
	@echo "  bench-step-profile Run step-load profile for one endpoint (ENDPOINT=...)"
	@echo "  analyze-step-profile Analyze one step-profile summary artifact"
	@echo "  compare-step-matrix Build cross-impl step analysis matrix"
	@echo "  bench-step-matrix Run cross-impl step-load orchestrator"
	@echo "  bench-step-matrix-dry Print step-load orchestrator command plan"
	@echo "  bench-matrix Run cross-impl benchmark orchestrator (IMPLS=$(IMPLS), ENDPOINTS=$(ENDPOINTS))"
	@echo "  bench-matrix-dry Print orchestrator commands without executing load"
	@echo "  bench-matrix-lasm-postgres Run LASM DB hot-path matrix in postgres mode (IMPLS=$(LASM_DB_MATRIX_IMPLS), ENDPOINTS=$(LASM_DB_MATRIX_ENDPOINTS))"
	@echo "  bench-matrix-lasm-postgres-dry Print LASM postgres DB matrix command plan"
	@echo "  bench-full Run fixed-target + step-load suites and publish combined report"
	@echo "  bench-full-dry Print full-suite command plan"
	@echo "  bench-full-lasm-postgres Run full suite in LASM postgres mode for DB hot-path endpoints"
	@echo "  bench-full-lasm-postgres-dry Print full LASM postgres suite command plan"
	@echo "  bench-alpha-postgres-suite Run standardized two-phase baseline+DB-hot postgres comparison suite"
	@echo "  bench-alpha-postgres-suite-dry Print standardized two-phase postgres comparison suite command plan"
	@echo "    - set BENCH_ALPHA_POSTGRES_RESET_BETWEEN_PHASES=true to reset Postgres benchmark tables before DB-hot phase"
	@echo "  bench-alpha-postgres-suite-local Run standardized postgres suite with repo local postgres infra orchestration"
	@echo "  bench-alpha-postgres-suite-local-dry Print local-infra postgres suite command plan without docker start/stop"
	@echo "  bench-full-saturation Run full suite plus LASM saturation lane (tunable via LASM_CAPACITY_* vars incl PROFILE/WARMUP_PATH); set FULL_LASM_INCLUDE_MODE_COMPARE=true to add mode compare lane"
	@echo "  bench-full-saturation-dry Print full-suite+saturation command plan"
	@echo "  bench-full-saturation-throughput Run saturation-enabled full suite with tuned throughput preset knobs"
	@echo "  bench-full-saturation-throughput-dry Print throughput preset full-suite+saturation plan"
	@echo "  bench-full-saturation-latency Run saturation-enabled full suite with latency-oriented preset knobs"
	@echo "  bench-full-saturation-latency-dry Print latency preset full-suite+saturation plan"
	@echo "  bench-full-saturation-presets Run throughput preset followed by latency preset"
	@echo "  bench-full-saturation-presets-dry Print both preset full-suite+saturation plans"
	@echo "  sec4-capacity-probe Run sec4 capacity probe (service build + load + peak RSS capture)"
	@echo "  lasm-cluster-capacity-probe Run sec4 LASM cluster capacity probe (sec4 run + load + peak RSS capture)"
	@echo "  lasm-cluster-mode-compare Run proxy-relay vs fixed reuse-port capacity comparison"
	@echo "  lasm-cluster-saturation-boost-matrix Run LASM cluster capacity probe matrix across saturation boost-step values"
	@echo "  lasm-cluster-saturation-boost-analyze Analyze LASM saturation boost matrix and pick recommended boost step"
	@echo "  lasm-cluster-saturation-boost-verify Run matrix + analysis and verify one recommended boost-step follow-up probe"
	@echo "  lasm-cluster-saturation-boost-summary Render markdown summary from saturation matrix + analysis (+ optional verify artifact)"
	@echo "  lasm-cluster-saturation-boost-bundle Run matrix + analysis + recommended verify + summary in one command"
	@echo "  artifact-manifest Build deterministic artifact manifest from results/"
	@echo "  verify-bundle Verify expected benchmark bundle artifacts for IMPLS/ENDPOINTS"
	@echo "  summarize    Convert raw wrk2 output to summary JSON"
	@echo "  report       Bundle summaries + env into one report JSON"
	@echo "  compare      Build endpoint comparison from report files"
	@echo "  compare-matrix Build comparison matrix across all endpoints"
	@echo "  analyze-matrix Analyze tail-latency/target-coverage signals from matrix"
	@echo "  publish-report Build markdown benchmark report from matrix + sec4 audit (+ optional saturation summary)"
	@echo "  test-scripts Run benchmark helper script tests"
	@echo "  test-services Run cross-impl service contract smoke checks"

db-up:
	docker compose -f docker-compose.yml up -d

db-down:
	docker compose -f docker-compose.yml down

db-schema:
	psql "postgresql://bench:bench@127.0.0.1:5432/bench" -f spec/db/schema.sql

env:
	scripts/capture_env.sh results/env.json

preflight:
	scripts/preflight.sh --impls "$(IMPLS)"

preflight-dry:
	scripts/preflight.sh --impls "$(IMPLS)" --dry-run-only

bench-ping:
	scripts/run_profile.sh $(IMPL) ping $(BASE_URL)

bench-decode:
	scripts/run_profile.sh $(IMPL) decode $(BASE_URL)

bench-users:
	scripts/run_profile.sh $(IMPL) users-post $(BASE_URL)

bench-users-get:
	scripts/run_profile.sh $(IMPL) users-get $(BASE_URL)

bench-profile:
	scripts/run_profile.sh $(IMPL) $(ENDPOINT) $(BASE_URL)

bench-step-profile:
	scripts/run_step_profile.sh $(IMPL) $(ENDPOINT) $(BASE_URL)

analyze-step-profile:
	scripts/analyze_step_profile.sh $(STEP_SUMMARY) $(STEP_ANALYSIS)

compare-step-matrix:
	scripts/compare_step_matrix.sh results/summaries $(STEP_MATRIX) "$(IMPLS)" "$(ENDPOINTS)"

bench-step-matrix:
	scripts/run_step_matrix.sh --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" $(LASM_DB_ADAPTER_FLAG) $(LASM_DB_POSTGRES_DSN_FILE_FLAG)

bench-step-matrix-dry:
	scripts/run_step_matrix.sh --dry-run --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" $(LASM_DB_ADAPTER_FLAG) $(LASM_DB_POSTGRES_DSN_FILE_FLAG)

bench-matrix:
	scripts/run_comparison_matrix.sh --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" $(LASM_DB_ADAPTER_FLAG) $(LASM_DB_POSTGRES_DSN_FILE_FLAG)

bench-matrix-dry:
	scripts/run_comparison_matrix.sh --dry-run --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" $(LASM_DB_ADAPTER_FLAG) $(LASM_DB_POSTGRES_DSN_FILE_FLAG)

bench-matrix-lasm-postgres:
	scripts/run_comparison_matrix.sh --impls "$(LASM_DB_MATRIX_IMPLS)" --endpoints "$(LASM_DB_MATRIX_ENDPOINTS)" --lasm-db-adapter postgres $(LASM_DB_POSTGRES_DSN_FILE_FLAG)

bench-matrix-lasm-postgres-dry:
	scripts/run_comparison_matrix.sh --dry-run --impls "$(LASM_DB_MATRIX_IMPLS)" --endpoints "$(LASM_DB_MATRIX_ENDPOINTS)" --lasm-db-adapter postgres $(LASM_DB_POSTGRES_DSN_FILE_FLAG)

bench-full:
	scripts/run_full_benchmark_suite.sh --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" $(LASM_DB_ADAPTER_FLAG) $(LASM_DB_POSTGRES_DSN_FILE_FLAG) $(LASM_MODE_COMPARE_FLAG)

bench-full-dry:
	scripts/run_full_benchmark_suite.sh --dry-run --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" $(LASM_DB_ADAPTER_FLAG) $(LASM_DB_POSTGRES_DSN_FILE_FLAG) $(LASM_MODE_COMPARE_FLAG)

bench-full-lasm-postgres:
	scripts/run_full_benchmark_suite.sh --impls "$(LASM_DB_MATRIX_IMPLS)" --endpoints "$(LASM_DB_MATRIX_ENDPOINTS)" --lasm-db-adapter postgres $(LASM_DB_POSTGRES_DSN_FILE_FLAG) $(LASM_MODE_COMPARE_FLAG)

bench-full-lasm-postgres-dry:
	scripts/run_full_benchmark_suite.sh --dry-run --impls "$(LASM_DB_MATRIX_IMPLS)" --endpoints "$(LASM_DB_MATRIX_ENDPOINTS)" --lasm-db-adapter postgres $(LASM_DB_POSTGRES_DSN_FILE_FLAG) $(LASM_MODE_COMPARE_FLAG)

bench-alpha-postgres-suite:
	scripts/run_alpha_postgres_comparison_suite.sh --base-impls "$(IMPLS)" --base-endpoints "$(ENDPOINTS)" --db-impls "$(LASM_DB_MATRIX_IMPLS)" --db-endpoints "$(LASM_DB_MATRIX_ENDPOINTS)" $(LASM_DB_POSTGRES_DSN_FILE_FLAG) $(ALPHA_POSTGRES_RESET_BETWEEN_PHASES_FLAG)

bench-alpha-postgres-suite-dry:
	scripts/run_alpha_postgres_comparison_suite.sh --dry-run --base-impls "$(IMPLS)" --base-endpoints "$(ENDPOINTS)" --db-impls "$(LASM_DB_MATRIX_IMPLS)" --db-endpoints "$(LASM_DB_MATRIX_ENDPOINTS)" $(LASM_DB_POSTGRES_DSN_FILE_FLAG) $(ALPHA_POSTGRES_RESET_BETWEEN_PHASES_FLAG)

bench-alpha-postgres-suite-local:
	scripts/run_alpha_postgres_comparison_suite_local.sh --base-impls "$(IMPLS)" --base-endpoints "$(ENDPOINTS)" --db-impls "$(LASM_DB_MATRIX_IMPLS)" --db-endpoints "$(LASM_DB_MATRIX_ENDPOINTS)" $(BENCH_LOCAL_POSTGRES_KEEP_UP_FLAG) $(BENCH_LOCAL_POSTGRES_RESET_FLAG) $(ALPHA_POSTGRES_RESET_BETWEEN_PHASES_FLAG)

bench-alpha-postgres-suite-local-dry:
	scripts/run_alpha_postgres_comparison_suite_local.sh --dry-run --base-impls "$(IMPLS)" --base-endpoints "$(ENDPOINTS)" --db-impls "$(LASM_DB_MATRIX_IMPLS)" --db-endpoints "$(LASM_DB_MATRIX_ENDPOINTS)" $(BENCH_LOCAL_POSTGRES_KEEP_UP_FLAG) $(BENCH_LOCAL_POSTGRES_RESET_FLAG) $(ALPHA_POSTGRES_RESET_BETWEEN_PHASES_FLAG)

bench-full-saturation:
	@if [ "$(FULL_SATURATION_SKIP_VERIFY)" = "true" ]; then \
		scripts/run_full_benchmark_suite.sh --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" $(LASM_DB_ADAPTER_FLAG) $(LASM_DB_POSTGRES_DSN_FILE_FLAG) --include-lasm-saturation --saturation-skip-verify --saturation-boost-steps "$(LASM_CAPACITY_BOOST_STEPS)" --saturation-profile "$(LASM_CAPACITY_PROFILE)" $(FULL_SATURATION_WARMUP_PATH_FLAG) --saturation-project-path "$(LASM_CAPACITY_PROJECT_PATH)" --saturation-target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --saturation-duration "$(LASM_CAPACITY_DURATION)" --saturation-threads "$(LASM_CAPACITY_THREADS)" --saturation-connections "$(LASM_CAPACITY_CONNECTIONS)" --saturation-cluster-relay-workers "$(LASM_CAPACITY_CLUSTER_RELAY_WORKERS)" --saturation-cluster-relay-queue "$(LASM_CAPACITY_CLUSTER_RELAY_QUEUE)" --saturation-cluster-accept-workers "$(LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS)" --saturation-cluster-relay-accept-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX)" --saturation-cluster-relay-pump-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX)" $(SATURATION_FIXED_REUSE_PORT_FLAG) $(LASM_MODE_COMPARE_FLAG); \
	else \
		scripts/run_full_benchmark_suite.sh --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" $(LASM_DB_ADAPTER_FLAG) $(LASM_DB_POSTGRES_DSN_FILE_FLAG) --include-lasm-saturation --saturation-boost-steps "$(LASM_CAPACITY_BOOST_STEPS)" --saturation-profile "$(LASM_CAPACITY_PROFILE)" $(FULL_SATURATION_WARMUP_PATH_FLAG) --saturation-project-path "$(LASM_CAPACITY_PROJECT_PATH)" --saturation-target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --saturation-duration "$(LASM_CAPACITY_DURATION)" --saturation-threads "$(LASM_CAPACITY_THREADS)" --saturation-connections "$(LASM_CAPACITY_CONNECTIONS)" --saturation-cluster-relay-workers "$(LASM_CAPACITY_CLUSTER_RELAY_WORKERS)" --saturation-cluster-relay-queue "$(LASM_CAPACITY_CLUSTER_RELAY_QUEUE)" --saturation-cluster-accept-workers "$(LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS)" --saturation-cluster-relay-accept-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX)" --saturation-cluster-relay-pump-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX)" $(SATURATION_FIXED_REUSE_PORT_FLAG) $(LASM_MODE_COMPARE_FLAG); \
	fi

bench-full-saturation-dry:
	@if [ "$(FULL_SATURATION_SKIP_VERIFY)" = "true" ]; then \
		scripts/run_full_benchmark_suite.sh --dry-run --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" $(LASM_DB_ADAPTER_FLAG) $(LASM_DB_POSTGRES_DSN_FILE_FLAG) --include-lasm-saturation --saturation-skip-verify --saturation-boost-steps "$(LASM_CAPACITY_BOOST_STEPS)" --saturation-profile "$(LASM_CAPACITY_PROFILE)" $(FULL_SATURATION_WARMUP_PATH_FLAG) --saturation-project-path "$(LASM_CAPACITY_PROJECT_PATH)" --saturation-target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --saturation-duration "$(LASM_CAPACITY_DURATION)" --saturation-threads "$(LASM_CAPACITY_THREADS)" --saturation-connections "$(LASM_CAPACITY_CONNECTIONS)" --saturation-cluster-relay-workers "$(LASM_CAPACITY_CLUSTER_RELAY_WORKERS)" --saturation-cluster-relay-queue "$(LASM_CAPACITY_CLUSTER_RELAY_QUEUE)" --saturation-cluster-accept-workers "$(LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS)" --saturation-cluster-relay-accept-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX)" --saturation-cluster-relay-pump-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX)" $(SATURATION_FIXED_REUSE_PORT_FLAG) $(LASM_MODE_COMPARE_FLAG); \
	else \
		scripts/run_full_benchmark_suite.sh --dry-run --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" $(LASM_DB_ADAPTER_FLAG) $(LASM_DB_POSTGRES_DSN_FILE_FLAG) --include-lasm-saturation --saturation-boost-steps "$(LASM_CAPACITY_BOOST_STEPS)" --saturation-profile "$(LASM_CAPACITY_PROFILE)" $(FULL_SATURATION_WARMUP_PATH_FLAG) --saturation-project-path "$(LASM_CAPACITY_PROJECT_PATH)" --saturation-target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --saturation-duration "$(LASM_CAPACITY_DURATION)" --saturation-threads "$(LASM_CAPACITY_THREADS)" --saturation-connections "$(LASM_CAPACITY_CONNECTIONS)" --saturation-cluster-relay-workers "$(LASM_CAPACITY_CLUSTER_RELAY_WORKERS)" --saturation-cluster-relay-queue "$(LASM_CAPACITY_CLUSTER_RELAY_QUEUE)" --saturation-cluster-accept-workers "$(LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS)" --saturation-cluster-relay-accept-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX)" --saturation-cluster-relay-pump-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX)" $(SATURATION_FIXED_REUSE_PORT_FLAG) $(LASM_MODE_COMPARE_FLAG); \
	fi

bench-full-saturation-throughput:
	@$(MAKE) bench-full-saturation \
		IMPLS="$(LASM_SATURATION_THROUGHPUT_IMPLS)" \
		ENDPOINTS="$(LASM_SATURATION_THROUGHPUT_ENDPOINTS)" \
		FULL_SATURATION_SKIP_VERIFY=true \
		LASM_CAPACITY_BOOST_STEPS="$(LASM_SATURATION_THROUGHPUT_BOOST_STEPS)" \
		LASM_CAPACITY_DURATION="$(LASM_SATURATION_THROUGHPUT_DURATION)" \
		LASM_CAPACITY_THREADS="$(LASM_SATURATION_THROUGHPUT_THREADS)" \
		LASM_CAPACITY_CONNECTIONS="$(LASM_SATURATION_THROUGHPUT_CONNECTIONS)" \
		LASM_CAPACITY_PROFILE="$(LASM_SATURATION_THROUGHPUT_PROFILE)" \
		LASM_CAPACITY_WARMUP_PATH="$(LASM_SATURATION_THROUGHPUT_WARMUP_PATH)" \
		LASM_CAPACITY_TARGET_REQUESTS="$(LASM_SATURATION_THROUGHPUT_TARGET_REQUESTS)" \
		LASM_CAPACITY_CLUSTER_RELAY_WORKERS="$(LASM_SATURATION_THROUGHPUT_RELAY_WORKERS)" \
		LASM_CAPACITY_CLUSTER_RELAY_QUEUE="$(LASM_SATURATION_THROUGHPUT_RELAY_QUEUE)" \
		LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX="$(LASM_SATURATION_THROUGHPUT_RELAY_PUMP_BATCH_MAX)" \
		LASM_CAPACITY_FIXED_REUSE_PORT_MODE="$(LASM_SATURATION_THROUGHPUT_FIXED_REUSE_PORT_MODE)"

bench-full-saturation-throughput-dry:
	@$(MAKE) bench-full-saturation-dry \
		IMPLS="$(LASM_SATURATION_THROUGHPUT_IMPLS)" \
		ENDPOINTS="$(LASM_SATURATION_THROUGHPUT_ENDPOINTS)" \
		FULL_SATURATION_SKIP_VERIFY=true \
		LASM_CAPACITY_BOOST_STEPS="$(LASM_SATURATION_THROUGHPUT_BOOST_STEPS)" \
		LASM_CAPACITY_DURATION="$(LASM_SATURATION_THROUGHPUT_DURATION)" \
		LASM_CAPACITY_THREADS="$(LASM_SATURATION_THROUGHPUT_THREADS)" \
		LASM_CAPACITY_CONNECTIONS="$(LASM_SATURATION_THROUGHPUT_CONNECTIONS)" \
		LASM_CAPACITY_PROFILE="$(LASM_SATURATION_THROUGHPUT_PROFILE)" \
		LASM_CAPACITY_WARMUP_PATH="$(LASM_SATURATION_THROUGHPUT_WARMUP_PATH)" \
		LASM_CAPACITY_TARGET_REQUESTS="$(LASM_SATURATION_THROUGHPUT_TARGET_REQUESTS)" \
		LASM_CAPACITY_CLUSTER_RELAY_WORKERS="$(LASM_SATURATION_THROUGHPUT_RELAY_WORKERS)" \
		LASM_CAPACITY_CLUSTER_RELAY_QUEUE="$(LASM_SATURATION_THROUGHPUT_RELAY_QUEUE)" \
		LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX="$(LASM_SATURATION_THROUGHPUT_RELAY_PUMP_BATCH_MAX)" \
		LASM_CAPACITY_FIXED_REUSE_PORT_MODE="$(LASM_SATURATION_THROUGHPUT_FIXED_REUSE_PORT_MODE)"

bench-full-saturation-latency:
	@$(MAKE) bench-full-saturation \
		IMPLS="$(LASM_SATURATION_LATENCY_IMPLS)" \
		ENDPOINTS="$(LASM_SATURATION_LATENCY_ENDPOINTS)" \
		FULL_SATURATION_SKIP_VERIFY=true \
		LASM_CAPACITY_BOOST_STEPS="$(LASM_SATURATION_LATENCY_BOOST_STEPS)" \
		LASM_CAPACITY_DURATION="$(LASM_SATURATION_LATENCY_DURATION)" \
		LASM_CAPACITY_THREADS="$(LASM_SATURATION_LATENCY_THREADS)" \
		LASM_CAPACITY_CONNECTIONS="$(LASM_SATURATION_LATENCY_CONNECTIONS)" \
		LASM_CAPACITY_PROFILE="$(LASM_SATURATION_LATENCY_PROFILE)" \
		LASM_CAPACITY_WARMUP_PATH="$(LASM_SATURATION_LATENCY_WARMUP_PATH)" \
		LASM_CAPACITY_TARGET_REQUESTS="$(LASM_SATURATION_LATENCY_TARGET_REQUESTS)" \
		LASM_CAPACITY_CLUSTER_RELAY_WORKERS="$(LASM_SATURATION_LATENCY_RELAY_WORKERS)" \
		LASM_CAPACITY_CLUSTER_RELAY_QUEUE="$(LASM_SATURATION_LATENCY_RELAY_QUEUE)" \
		LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX="$(LASM_SATURATION_LATENCY_RELAY_PUMP_BATCH_MAX)" \
		LASM_CAPACITY_FIXED_REUSE_PORT_MODE="$(LASM_SATURATION_LATENCY_FIXED_REUSE_PORT_MODE)"

bench-full-saturation-latency-dry:
	@$(MAKE) bench-full-saturation-dry \
		IMPLS="$(LASM_SATURATION_LATENCY_IMPLS)" \
		ENDPOINTS="$(LASM_SATURATION_LATENCY_ENDPOINTS)" \
		FULL_SATURATION_SKIP_VERIFY=true \
		LASM_CAPACITY_BOOST_STEPS="$(LASM_SATURATION_LATENCY_BOOST_STEPS)" \
		LASM_CAPACITY_DURATION="$(LASM_SATURATION_LATENCY_DURATION)" \
		LASM_CAPACITY_THREADS="$(LASM_SATURATION_LATENCY_THREADS)" \
		LASM_CAPACITY_CONNECTIONS="$(LASM_SATURATION_LATENCY_CONNECTIONS)" \
		LASM_CAPACITY_PROFILE="$(LASM_SATURATION_LATENCY_PROFILE)" \
		LASM_CAPACITY_WARMUP_PATH="$(LASM_SATURATION_LATENCY_WARMUP_PATH)" \
		LASM_CAPACITY_TARGET_REQUESTS="$(LASM_SATURATION_LATENCY_TARGET_REQUESTS)" \
		LASM_CAPACITY_CLUSTER_RELAY_WORKERS="$(LASM_SATURATION_LATENCY_RELAY_WORKERS)" \
		LASM_CAPACITY_CLUSTER_RELAY_QUEUE="$(LASM_SATURATION_LATENCY_RELAY_QUEUE)" \
		LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX="$(LASM_SATURATION_LATENCY_RELAY_PUMP_BATCH_MAX)" \
		LASM_CAPACITY_FIXED_REUSE_PORT_MODE="$(LASM_SATURATION_LATENCY_FIXED_REUSE_PORT_MODE)"

bench-full-saturation-presets:
	@$(MAKE) bench-full-saturation-throughput
	@$(MAKE) bench-full-saturation-latency

bench-full-saturation-presets-dry:
	@$(MAKE) bench-full-saturation-throughput-dry
	@$(MAKE) bench-full-saturation-latency-dry

sec4-capacity-probe:
	scripts/run_sec4_capacity_probe.sh --endpoint "$(CAPACITY_ENDPOINT)" --target-requests "$(CAPACITY_TARGET_REQUESTS)"

lasm-cluster-capacity-probe:
	scripts/run_lasm_cluster_capacity_probe.sh --project-path "$(LASM_CAPACITY_PROJECT_PATH)" --target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --profile "$(LASM_CAPACITY_PROFILE)" $(SATURATION_WARMUP_PATH_FLAG) --cluster-relay-workers "$(LASM_CAPACITY_CLUSTER_RELAY_WORKERS)" --cluster-relay-queue "$(LASM_CAPACITY_CLUSTER_RELAY_QUEUE)" --cluster-accept-workers "$(LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS)" --cluster-relay-accept-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX)" --cluster-relay-pump-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX)" $(FIXED_REUSE_PORT_FLAG)

lasm-cluster-mode-compare:
	scripts/run_lasm_cluster_mode_compare.sh --project-path "$(LASM_CAPACITY_PROJECT_PATH)" --target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --cluster-relay-workers "$(LASM_CAPACITY_CLUSTER_RELAY_WORKERS)" --cluster-relay-queue "$(LASM_CAPACITY_CLUSTER_RELAY_QUEUE)" --cluster-accept-workers "$(LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS)" --cluster-relay-accept-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX)" --cluster-relay-pump-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX)"

lasm-cluster-saturation-boost-matrix:
	scripts/run_lasm_cluster_saturation_boost_matrix.sh --project-path "$(LASM_CAPACITY_PROJECT_PATH)" --target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --profile "$(LASM_CAPACITY_PROFILE)" $(SATURATION_WARMUP_PATH_FLAG) --boost-steps "$(LASM_CAPACITY_BOOST_STEPS)" --cluster-relay-workers "$(LASM_CAPACITY_CLUSTER_RELAY_WORKERS)" --cluster-relay-queue "$(LASM_CAPACITY_CLUSTER_RELAY_QUEUE)" --cluster-accept-workers "$(LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS)" --cluster-relay-accept-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX)" --cluster-relay-pump-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX)" $(FIXED_REUSE_PORT_FLAG) --out "$(LASM_SATURATION_MATRIX)" --analysis-out "$(LASM_SATURATION_ANALYSIS)"

lasm-cluster-saturation-boost-analyze:
	scripts/analyze_lasm_cluster_saturation_boost_matrix.sh "$(LASM_SATURATION_MATRIX)" "$(LASM_SATURATION_ANALYSIS)"

lasm-cluster-saturation-boost-verify:
	scripts/run_lasm_cluster_saturation_boost_matrix.sh --project-path "$(LASM_CAPACITY_PROJECT_PATH)" --target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --profile "$(LASM_CAPACITY_PROFILE)" $(SATURATION_WARMUP_PATH_FLAG) --boost-steps "$(LASM_CAPACITY_BOOST_STEPS)" --cluster-relay-workers "$(LASM_CAPACITY_CLUSTER_RELAY_WORKERS)" --cluster-relay-queue "$(LASM_CAPACITY_CLUSTER_RELAY_QUEUE)" --cluster-accept-workers "$(LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS)" --cluster-relay-accept-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX)" --cluster-relay-pump-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX)" $(FIXED_REUSE_PORT_FLAG) --out "$(LASM_SATURATION_MATRIX)" --analysis-out "$(LASM_SATURATION_ANALYSIS)" --verify-recommended --verify-out "$(LASM_SATURATION_VERIFY_OUT)"

lasm-cluster-saturation-boost-summary:
	@if [ -n "$(LASM_SATURATION_VERIFY_INPUT)" ]; then \
		scripts/render_lasm_cluster_saturation_boost_summary.sh "$(LASM_SATURATION_MATRIX)" "$(LASM_SATURATION_ANALYSIS)" "$(LASM_SATURATION_SUMMARY)" "$(LASM_SATURATION_VERIFY_INPUT)"; \
	else \
		scripts/render_lasm_cluster_saturation_boost_summary.sh "$(LASM_SATURATION_MATRIX)" "$(LASM_SATURATION_ANALYSIS)" "$(LASM_SATURATION_SUMMARY)"; \
	fi

lasm-cluster-saturation-boost-bundle:
	scripts/run_lasm_cluster_saturation_boost_bundle.sh --project-path "$(LASM_CAPACITY_PROJECT_PATH)" --target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --profile "$(LASM_CAPACITY_PROFILE)" $(SATURATION_WARMUP_PATH_FLAG) --boost-steps "$(LASM_CAPACITY_BOOST_STEPS)" --cluster-relay-workers "$(LASM_CAPACITY_CLUSTER_RELAY_WORKERS)" --cluster-relay-queue "$(LASM_CAPACITY_CLUSTER_RELAY_QUEUE)" --cluster-accept-workers "$(LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS)" --cluster-relay-accept-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX)" --cluster-relay-pump-batch-max "$(LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX)" $(FIXED_REUSE_PORT_FLAG) --matrix-out "$(LASM_SATURATION_MATRIX)" --analysis-out "$(LASM_SATURATION_ANALYSIS)" --verify-out "$(LASM_SATURATION_VERIFY_OUT)" --summary-out "$(LASM_SATURATION_SUMMARY)"

artifact-manifest:
	scripts/build_artifact_manifest.sh results "$(ARTIFACT_MANIFEST)"

verify-bundle:
	scripts/verify_benchmark_bundle.sh results "$(IMPLS)" "$(ENDPOINTS)"

summarize:
	scripts/wrk2_summary.sh results/raw/$(IMPL)-ping.txt $(IMPL) ping 10000 results/summaries/$(IMPL)-ping.json

report:
	scripts/build_report.sh $(IMPL) results results/summaries/$(IMPL)-report.json

compare:
	scripts/compare_reports.sh results/summaries ping results/summaries/compare-ping.json

compare-matrix:
	scripts/compare_matrix.sh results/summaries results/summaries/compare-matrix.json "$(IMPLS)"

analyze-matrix:
	scripts/analyze_matrix.sh results/summaries/compare-matrix.json results/summaries/analysis.json

publish-report:
	@if [ -n "$(SATURATION_SUMMARY)" ]; then \
		scripts/publish_report.sh results/summaries/compare-matrix.json results/benchmark-report.md ../baselines/sec-audit/default-secure-prod.hello.json results/summaries/analysis.json "" "$(SATURATION_SUMMARY)"; \
	else \
		scripts/publish_report.sh results/summaries/compare-matrix.json results/benchmark-report.md ../baselines/sec-audit/default-secure-prod.hello.json results/summaries/analysis.json; \
	fi

test-scripts:
	scripts/test_run_sec4_capacity_probe.sh
	scripts/test_run_lasm_cluster_capacity_probe.sh
	scripts/test_run_lasm_cluster_mode_compare.sh
	scripts/test_run_lasm_cluster_saturation_boost_matrix.sh
	scripts/test_run_lasm_cluster_saturation_boost_bundle.sh
	scripts/test_analyze_lasm_cluster_saturation_boost_matrix.sh
	scripts/test_render_lasm_cluster_saturation_boost_summary.sh
	scripts/test_wrk2_summary.sh
	scripts/test_run_profile.sh
	scripts/test_build_report.sh
	scripts/test_compare_reports.sh
	scripts/test_compare_reports_contract.sh
	scripts/test_compare_matrix.sh
	scripts/test_compare_matrix_contract.sh
	scripts/test_compare_step_matrix.sh
	scripts/test_analyze_step_profile.sh
	scripts/test_analyze_matrix.sh
	scripts/test_publish_report.sh
	scripts/test_run_comparison_matrix.sh
	scripts/test_run_step_profile.sh
	scripts/test_run_step_matrix.sh
	scripts/test_run_full_benchmark_suite.sh
	scripts/test_run_alpha_postgres_comparison_suite.sh
	scripts/test_run_alpha_postgres_comparison_suite_local.sh
	scripts/test_build_artifact_manifest.sh
	scripts/test_verify_benchmark_bundle.sh
	scripts/test_preflight.sh
	scripts/test_makefile_profile_targets.sh
	scripts/test_bench_full_saturation_throughput_profile.sh
	scripts/test_bench_full_saturation_latency_profile.sh
	scripts/test_bench_full_saturation_presets_profile.sh
	scripts/test_test_service_contracts.sh

test-services:
	scripts/test_service_contracts.sh
