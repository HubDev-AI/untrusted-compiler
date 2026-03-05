.PHONY: check run-records run-sqlite run-postgres smoke smoke-records smoke-sqlite smoke-postgres smoke-help clean-artifacts

DB_BASE ?= $(CURDIR)/.lasm-db
PORT ?= 8088
TIMEOUT_MS ?= 5000
SCRIPT := ./scripts/run-smoke.sh

check:
	cargo run -p sec4 -- check --path $(CURDIR)

run-records:
	SEC4_ALPHA_FULL_DB_BASE="$(DB_BASE)" \
	SEC4_ALPHA_FULL_DB_ADAPTER=records \
	SEC4_ALPHA_FULL_TIMEOUT_MS="$(TIMEOUT_MS)" \
	cargo run -p sec4 -- run \
		--path $(CURDIR) \
		--backend lasm \
		--db-base "$(DB_BASE)" \
		--port $(PORT)

run-sqlite:
	SEC4_ALPHA_FULL_DB_BASE="$(DB_BASE)" \
	SEC4_ALPHA_FULL_DB_ADAPTER=sqlite \
	SEC4_ALPHA_FULL_TIMEOUT_MS="$(TIMEOUT_MS)" \
	cargo run -p sec4 -- run \
		--path $(CURDIR) \
		--backend lasm \
		--db-base "$(DB_BASE)" \
		--db-adapter sqlite \
		--port $(PORT)

run-postgres:
	SEC4_ALPHA_FULL_DB_BASE="$(DB_BASE)" \
	SEC4_ALPHA_FULL_DB_ADAPTER=postgres \
	SEC4_ALPHA_FULL_TIMEOUT_MS="$(TIMEOUT_MS)" \
	cargo run -p sec4 -- run \
		--path $(CURDIR) \
		--backend lasm \
		--db-base "$(DB_BASE)" \
		--db-adapter postgres \
		--port $(PORT)

smoke: smoke-records

smoke-records:
	SEC4_ALPHA_FULL_DB_BASE="$(DB_BASE)" \
	SEC4_ALPHA_FULL_DB_ADAPTER=records \
	SEC4_ALPHA_FULL_TIMEOUT_MS="$(TIMEOUT_MS)" \
	$(SCRIPT)

smoke-sqlite:
	SEC4_ALPHA_FULL_DB_BASE="$(DB_BASE)" \
	SEC4_ALPHA_FULL_DB_ADAPTER=sqlite \
	SEC4_ALPHA_FULL_TIMEOUT_MS="$(TIMEOUT_MS)" \
	$(SCRIPT)

smoke-postgres:
	SEC4_ALPHA_FULL_DB_BASE="$(DB_BASE)" \
	SEC4_ALPHA_FULL_DB_ADAPTER=postgres \
	SEC4_ALPHA_FULL_TIMEOUT_MS="$(TIMEOUT_MS)" \
	$(SCRIPT)

smoke-help:
	$(SCRIPT) --help

clean-artifacts:
	[ -d "$(DB_BASE)" ] && rm -rf "$(DB_BASE)"
