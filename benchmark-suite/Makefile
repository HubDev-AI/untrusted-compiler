.PHONY: help db-up db-down db-schema env bench-ping bench-decode bench-users bench-profile bench-matrix bench-matrix-dry summarize report compare compare-matrix analyze-matrix publish-report test-scripts

IMPL ?= ailang
BASE_URL ?= http://127.0.0.1:8080
ENDPOINT ?= ping

help:
	@echo "Targets:"
	@echo "  db-up        Start benchmark postgres"
	@echo "  db-down      Stop benchmark postgres"
	@echo "  db-schema    Apply benchmark schema"
	@echo "  env          Capture environment metadata"
	@echo "  bench-ping   Run ping benchmark (requires service at $(BASE_URL))"
	@echo "  bench-decode Run decode benchmark (requires service at $(BASE_URL))"
	@echo "  bench-users  Run users write benchmark (requires service at $(BASE_URL))"
	@echo "  bench-profile Run one named profile via script (ENDPOINT=ping|decode|users-post)"
	@echo "  bench-matrix Run cross-impl benchmark orchestrator (node,go,rust by default)"
	@echo "  bench-matrix-dry Print orchestrator commands without executing load"
	@echo "  summarize    Convert raw wrk2 output to summary JSON"
	@echo "  report       Bundle summaries + env into one report JSON"
	@echo "  compare      Build endpoint comparison from report files"
	@echo "  compare-matrix Build comparison matrix across all endpoints"
	@echo "  analyze-matrix Analyze tail-latency/target-coverage signals from matrix"
	@echo "  publish-report Build markdown benchmark report from matrix + sec.audit"
	@echo "  test-scripts Run benchmark helper script tests"

db-up:
	docker compose -f docker-compose.yml up -d

db-down:
	docker compose -f docker-compose.yml down

db-schema:
	psql "postgresql://bench:bench@127.0.0.1:5432/bench" -f spec/db/schema.sql

env:
	scripts/capture_env.sh results/env.json

bench-ping:
	wrk2 --latency -t4 -c64 -d60s -R10000 $(BASE_URL)/ping | tee results/raw/$(IMPL)-ping.txt

bench-decode:
	wrk2 --latency -t4 -c64 -d60s -R2000 -s load/wrk2/post_decode.lua $(BASE_URL) | tee results/raw/$(IMPL)-decode.txt

bench-users:
	wrk2 --latency -t4 -c64 -d60s -R500 -s load/wrk2/post_users.lua $(BASE_URL) | tee results/raw/$(IMPL)-users-post.txt

bench-profile:
	scripts/run_profile.sh $(IMPL) $(ENDPOINT) $(BASE_URL)

bench-matrix:
	scripts/run_comparison_matrix.sh

bench-matrix-dry:
	scripts/run_comparison_matrix.sh --dry-run

summarize:
	scripts/wrk2_summary.sh results/raw/$(IMPL)-ping.txt $(IMPL) ping 10000 results/summaries/$(IMPL)-ping.json

report:
	scripts/build_report.sh $(IMPL) results results/summaries/$(IMPL)-report.json

compare:
	scripts/compare_reports.sh results/summaries ping results/summaries/compare-ping.json

compare-matrix:
	scripts/compare_matrix.sh results/summaries results/summaries/compare-matrix.json

analyze-matrix:
	scripts/analyze_matrix.sh results/summaries/compare-matrix.json results/summaries/analysis.json

publish-report:
	scripts/publish_report.sh results/summaries/compare-matrix.json results/benchmark-report.md ../baselines/sec-audit/default-secure-prod.hello.json

test-scripts:
	scripts/test_wrk2_summary.sh
	scripts/test_run_profile.sh
	scripts/test_build_report.sh
	scripts/test_compare_reports.sh
	scripts/test_compare_matrix.sh
	scripts/test_analyze_matrix.sh
	scripts/test_publish_report.sh
	scripts/test_run_comparison_matrix.sh
