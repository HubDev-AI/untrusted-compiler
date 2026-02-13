.PHONY: help db-up db-down db-schema env bench-ping bench-decode bench-users

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

db-up:
	docker compose -f benchmark-suite/docker-compose.yml up -d

db-down:
	docker compose -f benchmark-suite/docker-compose.yml down

db-schema:
	psql "postgresql://bench:bench@127.0.0.1:5432/bench" -f benchmark-suite/spec/db/schema.sql

env:
	benchmark-suite/scripts/capture_env.sh benchmark-suite/results/env.json

bench-ping:
	wrk2 -t4 -c64 -d60s -R10000 $(BASE_URL)/ping | tee benchmark-suite/results/raw/$(IMPL)-ping.txt

bench-decode:
	wrk2 -t4 -c64 -d60s -R2000 -s benchmark-suite/load/wrk2/post_decode.lua $(BASE_URL) | tee benchmark-suite/results/raw/$(IMPL)-decode.txt

bench-users:
	wrk2 -t4 -c64 -d60s -R500 -s benchmark-suite/load/wrk2/post_users.lua $(BASE_URL) | tee benchmark-suite/results/raw/$(IMPL)-users-post.txt
