.PHONY: help db-up db-down db-schema env preflight preflight-dry bench-ping bench-decode bench-users bench-users-get bench-profile bench-step-profile analyze-step-profile compare-step-matrix bench-step-matrix bench-step-matrix-dry bench-matrix bench-matrix-dry bench-full bench-full-dry artifact-manifest summarize report compare compare-matrix analyze-matrix publish-report test-scripts test-services

IMPL ?= ailang
IMPLS ?= ailang,node,go,rust
ENDPOINTS ?= ping,decode,users-post,users-get
BASE_URL ?= http://127.0.0.1:8080
ENDPOINT ?= ping
STEP_SUMMARY ?= results/summaries/$(IMPL)-$(ENDPOINT)-step.json
STEP_ANALYSIS ?= results/summaries/$(IMPL)-$(ENDPOINT)-step-analysis.json
STEP_MATRIX ?= results/summaries/step-matrix.json
ARTIFACT_MANIFEST ?= results/artifact-manifest.json

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
	@echo "  artifact-manifest Build deterministic artifact manifest from results/"
	@echo "  summarize    Convert raw wrk2 output to summary JSON"
	@echo "  report       Bundle summaries + env into one report JSON"
	@echo "  compare      Build endpoint comparison from report files"
	@echo "  compare-matrix Build comparison matrix across all endpoints"
	@echo "  analyze-matrix Analyze tail-latency/target-coverage signals from matrix"
	@echo "  publish-report Build markdown benchmark report from matrix + sec.audit"
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
	wrk2 --latency -t4 -c64 -d60s -R10000 $(BASE_URL)/ping | tee results/raw/$(IMPL)-ping.txt

bench-decode:
	wrk2 --latency -t4 -c64 -d60s -R2000 -s load/wrk2/post_decode.lua $(BASE_URL) | tee results/raw/$(IMPL)-decode.txt

bench-users:
	wrk2 --latency -t4 -c64 -d60s -R500 -s load/wrk2/post_users.lua $(BASE_URL) | tee results/raw/$(IMPL)-users-post.txt

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

artifact-manifest:
	scripts/build_artifact_manifest.sh results "$(ARTIFACT_MANIFEST)"

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
	scripts/publish_report.sh results/summaries/compare-matrix.json results/benchmark-report.md ../baselines/sec-audit/default-secure-prod.hello.json results/summaries/analysis.json

test-scripts:
	scripts/test_wrk2_summary.sh
	scripts/test_run_profile.sh
	scripts/test_build_report.sh
	scripts/test_compare_reports.sh
	scripts/test_compare_matrix.sh
	scripts/test_compare_step_matrix.sh
	scripts/test_analyze_step_profile.sh
	scripts/test_analyze_matrix.sh
	scripts/test_publish_report.sh
	scripts/test_run_comparison_matrix.sh
	scripts/test_run_step_profile.sh
	scripts/test_run_step_matrix.sh
	scripts/test_run_full_benchmark_suite.sh
	scripts/test_build_artifact_manifest.sh
	scripts/test_preflight.sh
	scripts/test_test_service_contracts.sh

test-services:
	scripts/test_service_contracts.sh
