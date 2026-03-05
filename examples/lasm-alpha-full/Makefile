.PHONY: check run-records run-sqlite run-postgres smoke smoke-records smoke-sqlite smoke-postgres smoke-help bench-workbench bench-workbench-fast bench-workbench-full bench-workbench-full-fast clean-artifacts

DB_BASE ?= $(CURDIR)/.lasm-db
PORT ?= 8088
TIMEOUT_MS ?= 5000
SCRIPT := ./scripts/run-smoke.sh
WORKBENCH_MATRIX_SCRIPT := ../../benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh
WORKBENCH_FULL_SCRIPT := ../../benchmark-suite/scripts/run_workbench_full_benchmark_suite_local.sh
WORKBENCH_IMPLS ?= sec4,sec4-lasm,node,go,rust
WORKBENCH_ENDPOINTS ?= wb-tasks-post,wb-tasks-with-comment,wb-task-comment-post,wb-task-get,wb-tasks-list

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

bench-workbench:
	$(WORKBENCH_MATRIX_SCRIPT) \
		--impls "$(WORKBENCH_IMPLS)" \
		--endpoints "$(WORKBENCH_ENDPOINTS)"

bench-workbench-fast:
	BENCH_DURATION=10s \
	BENCH_CONNECTIONS=32 \
	BENCH_THREADS=4 \
	$(WORKBENCH_MATRIX_SCRIPT) \
		--impls "$(WORKBENCH_IMPLS)" \
		--endpoints "$(WORKBENCH_ENDPOINTS)"

bench-workbench-full:
	$(WORKBENCH_FULL_SCRIPT) \
		--impls "$(WORKBENCH_IMPLS)" \
		--endpoints "$(WORKBENCH_ENDPOINTS)"

bench-workbench-full-fast:
	BENCH_DURATION=10s \
	BENCH_CONNECTIONS=32 \
	BENCH_THREADS=4 \
	BENCH_STEP_DURATION=8s \
	BENCH_STEP_CONNECTIONS=32 \
	BENCH_STEP_THREADS=4 \
	BENCH_STEP_MAX_RATE=300 \
	$(WORKBENCH_FULL_SCRIPT) \
		--impls "$(WORKBENCH_IMPLS)" \
		--endpoints "$(WORKBENCH_ENDPOINTS)"

clean-artifacts:
	[ -d "$(DB_BASE)" ] && rm -rf "$(DB_BASE)"
