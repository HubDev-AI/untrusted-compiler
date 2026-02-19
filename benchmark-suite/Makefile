.PHONY: help db-up db-down db-schema env preflight preflight-dry bench-ping bench-decode bench-users bench-users-get bench-profile bench-step-profile analyze-step-profile compare-step-matrix bench-step-matrix bench-step-matrix-dry bench-matrix bench-matrix-dry bench-full bench-full-dry bench-full-saturation bench-full-saturation-dry sec4-capacity-probe lasm-cluster-capacity-probe lasm-cluster-saturation-boost-matrix lasm-cluster-saturation-boost-analyze lasm-cluster-saturation-boost-verify lasm-cluster-saturation-boost-summary lasm-cluster-saturation-boost-bundle artifact-manifest verify-bundle summarize report compare compare-matrix analyze-matrix publish-report test-scripts test-services

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
LASM_CAPACITY_BOOST_STEPS ?= 2,4,6
LASM_SATURATION_MATRIX ?= results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json
LASM_SATURATION_ANALYSIS ?= results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json
LASM_SATURATION_VERIFY_OUT ?= results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json
LASM_SATURATION_SUMMARY ?= results/summaries/sec4-lasm-cluster-saturation-boost-summary.md
LASM_SATURATION_VERIFY_INPUT ?=
SATURATION_SUMMARY ?=
FULL_SATURATION_SKIP_VERIFY ?= false

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
	@echo "  bench-profile Run one named profile via script (ENDPOINT=ping|decode|users-post|users-get)"
	@echo "  bench-step-profile Run step-load profile for one endpoint (ENDPOINT=...)"
	@echo "  analyze-step-profile Analyze one step-profile summary artifact"
	@echo "  compare-step-matrix Build cross-impl step analysis matrix"
	@echo "  bench-step-matrix Run cross-impl step-load orchestrator"
	@echo "  bench-step-matrix-dry Print step-load orchestrator command plan"
	@echo "  bench-matrix Run cross-impl benchmark orchestrator (IMPLS=$(IMPLS), ENDPOINTS=$(ENDPOINTS))"
	@echo "  bench-matrix-dry Print orchestrator commands without executing load"
	@echo "  bench-full Run fixed-target + step-load suites and publish combined report"
	@echo "  bench-full-dry Print full-suite command plan"
	@echo "  bench-full-saturation Run full suite plus optional LASM saturation bundle lane"
	@echo "  bench-full-saturation-dry Print full-suite+saturation command plan"
	@echo "  sec4-capacity-probe Run sec4 capacity probe (service build + load + peak RSS capture)"
	@echo "  lasm-cluster-capacity-probe Run sec4 LASM cluster capacity probe (sec4 run + load + peak RSS capture)"
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
	scripts/run_step_matrix.sh --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)"

bench-step-matrix-dry:
	scripts/run_step_matrix.sh --dry-run --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)"

bench-matrix:
	scripts/run_comparison_matrix.sh --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)"

bench-matrix-dry:
	scripts/run_comparison_matrix.sh --dry-run --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)"

bench-full:
	scripts/run_full_benchmark_suite.sh --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)"

bench-full-dry:
	scripts/run_full_benchmark_suite.sh --dry-run --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)"

bench-full-saturation:
	@if [ "$(FULL_SATURATION_SKIP_VERIFY)" = "true" ]; then \
		scripts/run_full_benchmark_suite.sh --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" --include-lasm-saturation --saturation-skip-verify; \
	else \
		scripts/run_full_benchmark_suite.sh --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" --include-lasm-saturation; \
	fi

bench-full-saturation-dry:
	@if [ "$(FULL_SATURATION_SKIP_VERIFY)" = "true" ]; then \
		scripts/run_full_benchmark_suite.sh --dry-run --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" --include-lasm-saturation --saturation-skip-verify; \
	else \
		scripts/run_full_benchmark_suite.sh --dry-run --impls "$(IMPLS)" --endpoints "$(ENDPOINTS)" --include-lasm-saturation; \
	fi

sec4-capacity-probe:
	scripts/run_sec4_capacity_probe.sh --endpoint "$(CAPACITY_ENDPOINT)" --target-requests "$(CAPACITY_TARGET_REQUESTS)"

lasm-cluster-capacity-probe:
	scripts/run_lasm_cluster_capacity_probe.sh --project-path "$(LASM_CAPACITY_PROJECT_PATH)" --target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)"

lasm-cluster-saturation-boost-matrix:
	scripts/run_lasm_cluster_saturation_boost_matrix.sh --project-path "$(LASM_CAPACITY_PROJECT_PATH)" --target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --boost-steps "$(LASM_CAPACITY_BOOST_STEPS)" --out "$(LASM_SATURATION_MATRIX)" --analysis-out "$(LASM_SATURATION_ANALYSIS)"

lasm-cluster-saturation-boost-analyze:
	scripts/analyze_lasm_cluster_saturation_boost_matrix.sh "$(LASM_SATURATION_MATRIX)" "$(LASM_SATURATION_ANALYSIS)"

lasm-cluster-saturation-boost-verify:
	scripts/run_lasm_cluster_saturation_boost_matrix.sh --project-path "$(LASM_CAPACITY_PROJECT_PATH)" --target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --boost-steps "$(LASM_CAPACITY_BOOST_STEPS)" --out "$(LASM_SATURATION_MATRIX)" --analysis-out "$(LASM_SATURATION_ANALYSIS)" --verify-recommended --verify-out "$(LASM_SATURATION_VERIFY_OUT)"

lasm-cluster-saturation-boost-summary:
	@if [ -n "$(LASM_SATURATION_VERIFY_INPUT)" ]; then \
		scripts/render_lasm_cluster_saturation_boost_summary.sh "$(LASM_SATURATION_MATRIX)" "$(LASM_SATURATION_ANALYSIS)" "$(LASM_SATURATION_SUMMARY)" "$(LASM_SATURATION_VERIFY_INPUT)"; \
	else \
		scripts/render_lasm_cluster_saturation_boost_summary.sh "$(LASM_SATURATION_MATRIX)" "$(LASM_SATURATION_ANALYSIS)" "$(LASM_SATURATION_SUMMARY)"; \
	fi

lasm-cluster-saturation-boost-bundle:
	scripts/run_lasm_cluster_saturation_boost_bundle.sh --project-path "$(LASM_CAPACITY_PROJECT_PATH)" --target-requests "$(LASM_CAPACITY_TARGET_REQUESTS)" --boost-steps "$(LASM_CAPACITY_BOOST_STEPS)" --matrix-out "$(LASM_SATURATION_MATRIX)" --analysis-out "$(LASM_SATURATION_ANALYSIS)" --verify-out "$(LASM_SATURATION_VERIFY_OUT)" --summary-out "$(LASM_SATURATION_SUMMARY)"

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
	scripts/test_build_artifact_manifest.sh
	scripts/test_verify_benchmark_bundle.sh
	scripts/test_preflight.sh
	scripts/test_makefile_profile_targets.sh
	scripts/test_test_service_contracts.sh

test-services:
	scripts/test_service_contracts.sh
