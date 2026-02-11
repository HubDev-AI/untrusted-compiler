# 11 CLI and Build Lifecycle (M0)

## Commands exposed now

- `ailang build --path <project>`
- `ailang run --path <project>`
- `ailang check --path <project>`
- `ailang test --path <project>`
- `ailang fmt --path <project>`
- `ailang lint --path <project>`
- `ailang sec audit --path <project> [--format text|json] [--fail-on 'risk>=HIGH']`

## Command behavior in M0

- `check`: validate `ailang.toml` and entry file path/extension.
- `build`: run `check` validations + write lockfile stub (`ailang.lock`).
- `run`/`test`/`fmt`/`lint`: placeholders with deterministic messages and basic validation.
- `sec audit`: validates project + semantic checks, emits `build/security_map.json`, then renders deterministic posture findings.

## Build flow (current)

1. Load and parse `ailang.toml`.
2. Validate package and entry fields.
3. Validate entry file exists and uses `.ai` extension.
4. On `build`, write `ailang.lock` stub.
5. Return success/failure via diagnostics.

## Error reporting style

Diagnostics include:
- severity
- stable code (`M0001`, `M0101`, etc.)
- message
- file/line/column span
- optional notes

Color output is enabled for terminal readability.
