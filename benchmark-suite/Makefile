.PHONY: help db-up db-down db-schema env bench-ping bench-decode bench-users summarize test-scripts

IMPL ?= ailang
BASE_URL ?= http://127.0.0.1:8080

help:
	@echo "Targets:"
	@echo "  db-up        Start benchmark postgres"
	@echo "  db-down      Stop benchmark postgres"
	@echo "  db-schema    Apply benchmark schema"
	@echo "  env          Capture environment metadata"
	@echo "  bench-ping   Run ping benchmark (requires service at $(BASE_URL))"
	@echo "  bench-decode Run decode benchmark (requires service at $(BASE_URL))"
	@echo "  bench-users  Run users write benchmark (requires service at $(BASE_URL))"
	@echo "  summarize    Convert raw wrk2 output to summary JSON"
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

summarize:
	scripts/wrk2_summary.sh results/raw/$(IMPL)-ping.txt $(IMPL) ping 10000 results/summaries/$(IMPL)-ping.json

test-scripts:
	scripts/test_wrk2_summary.sh
