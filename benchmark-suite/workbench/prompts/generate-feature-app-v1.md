# Prompt: Generate Feature App v1 Service

Use this prompt to generate one backend implementation for the workbench contract.

## Prompt Body

You are implementing a backend service for benchmark parity.

Target backend: `<TARGET_BACKEND>`
Project root: `<TARGET_SERVICE_PATH>`

Read and follow exactly:

1. `benchmark-suite/workbench/spec/feature-app-v1.md`
2. `benchmark-suite/spec/endpoints.md` (for envelope/parity style where relevant)

Rules:

1. Implement all required endpoints from the workbench spec.
2. Use real Postgres access for DB routes (no in-memory fake DB for benchmark mode).
3. Enforce auth contract exactly (`Authorization: Bearer token123`).
4. Return deterministic success/error envelope shape exactly as specified.
5. Ensure deterministic ordering for list endpoint.
6. Include a runnable startup entrypoint binding to `127.0.0.1:${PORT}`.
7. Implement the alpha wire-format contract:
   - `params`, `task_params`, and `comment_params` query-string JSON arrays as defined in the spec.
8. Add a minimal smoke script that:
   - starts service,
   - calls one write endpoint and one read endpoint,
   - validates status code and key response fields.
9. Do not add placeholder TODO paths for required flows.
10. Keep implementation minimal and benchmark-focused (no extra features).

Deliverables:

1. Source files under `<TARGET_SERVICE_PATH>`
2. `README.md` with run/smoke commands
3. `smoke.sh`

Acceptance checklist:

1. `POST /wb/tasks` -> `201` and deterministic success envelope.
2. `POST /wb/tasks/with-comment` performs transaction semantics.
3. `GET /wb/tasks/:id` returns `404 TASK.NOT_FOUND` when absent.
4. `GET /wb/tasks` supports filters/pagination and deterministic ordering.
5. Auth-protected endpoints reject missing auth with `401 AUTH.REQUIRED`.
