# M39 - Promote Composition Contract Analyzer Implementation

## What Was Added

Implemented the Composition Contract Analyzer path used by `sec4 promote` dry-run planning.
This adds deterministic contract checks that keep project structure promotion-safe before rewrites execute.

Primary files:

1. `compiler/sec4-core/src/composition.rs`
2. `compiler/sec4-core/src/lib.rs`
3. `compiler/sec4-core/src/project.rs`
4. `compiler/sec4-cli/src/main.rs`
5. `compiler/sec4-cli/tests/commands.rs`

## Behavior

### Composition boundary checks

Promotion now validates a minimum safe architecture envelope before any apply step:

- `repo.*` imports are now required only in the composition root `src/main.ut`.
  - Violations emit `PROMOTE.P9401`.
- Domain modules (non-root modules outside `repo.*`) are no longer allowed to call server-only roots directly.
  - Violations emit `PROMOTE.P9404`.
  - Forbidden roots currently include:
    - `localdb`
    - `db`
    - `net`
    - `auth`
    - `csrf`
    - `http`
    - `httpClient`
    - `secrets`

### Repo adapter parity checks

`repo.browser_*` and `repo.server_*` adapters are compared by method name to guarantee a complete migration surface.

- Browser-only adapter methods emit `PROMOTE.P9402`.
- Server-only adapter methods emit `PROMOTE.P9403`.

### Stable diagnostics model

1. Checks are deterministic and include stable source line/file reporting.
2. Parse failures in module passes are folded into promotion preconditions as blocking diagnostics.
3. Analyzer output is merged into `promote` dry-run preconditions with existing non-blocking/ blocking behavior rules.

## Why

This is the first gate that stops projects from entering promotion when their domain/repo split is structurally unsafe.

Without it, compile-time success at source time can still leave runtime migrations impossible or unsafe:

- domain logic directly coupled to host-only APIs,
- adapter surface mismatch across browser/server sides,
- adapter imports spread outside composition root and rewrites becoming partial or wrong.

## Validation

1. `cargo test -p sec4 --test commands promote_dry_run_blocks_domain_import_of_repo_from_non_root_module`
2. `cargo test -p sec4 --test commands promote_dry_run_blocks_repo_adapter_parity_mismatch`
3. `cargo test -p sec4 --test commands promote_dry_run_blocks_domain_module_dependency_calls`
