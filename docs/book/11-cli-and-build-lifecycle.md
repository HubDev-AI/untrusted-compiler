# 11 CLI and Build Lifecycle (M0)

## Commands exposed now

- `sec4 build --path <project>`
- `sec4 run --path <project>`
- `sec4 check --path <project>`
- `sec4 test --path <project>`
- `sec4 fmt --path <project>`
- `sec4 lint --path <project>`
- `sec4 audit --path <project> [--format text|json] [--fail-on 'risk>=HIGH']`
- `sec4 explain <ERROR_CODE>`
- `sec4 gate --path <project> [--format text|json] [--fail-on 'risk>=HIGH']`

## Command behavior in M0

- `check`: validate `sec4.toml` and entry file path/extension.
- `build`: run `check` validations + write lockfile stub (`sec4.lock`).
- `run`/`test`/`fmt`/`lint`: placeholders with deterministic messages and basic validation.
- `audit`: validates project + semantic checks, emits `build/security_map.json`, then renders deterministic posture findings.
- `explain`: prints deterministic guidance entrypoint for a diagnostic code.
- `gate`: runs policy/security gate checks (threshold-oriented wrapper over audit flow).

## Build flow (current)

1. Load and parse `sec4.toml`.
2. Validate package and entry fields.
3. Validate entry file exists and uses `.ut` extension.
4. On `build`, write `sec4.lock` stub.
5. Return success/failure via diagnostics.

## Error reporting style

Diagnostics include:
- severity
- stable code (`M0001`, `M0101`, etc.)
- message
- file/line/column span
- optional notes

Color output is enabled for terminal readability.
