# Untrusted<T> Zed Extension

This directory contains the official Zed integration for Untrusted<T>.

## What it provides

- `extension.toml`:
  - registers `Untrusted<T>` grammar metadata,
  - registers `sec4audit-lsp` language server wiring.
- `languages/untrusted/config.toml`:
  - language name/grammar/file suffix/comment/bracket config.
- `src/lib.rs`:
  - resolves and launches `sec4audit-language-server --stdio` for Zed LSP requests.

## Feature surface (from LSP)

- diagnostics (`publishDiagnostics`)
- go to definition / hover
- references / rename
- quickfix code actions
- document formatting (`textDocument/formatting`)

## Local smoke project

Use:

- `examples/zed-plugin-smoke`

It includes multi-file `.ut` modules plus playground files for diagnostics and formatter checks.

Run the deterministic smoke runner:

```bash
scripts/run-zed-plugin-smoke.sh
```

## Local install / update / rollback

Use the operator helper script:

```bash
# inspect resolved install paths and current state
scripts/manage-zed-extension-local.sh status

# install current repo extension (default: copy mode)
scripts/manage-zed-extension-local.sh install

# update existing local install (backs up previous install first)
scripts/manage-zed-extension-local.sh update

# rollback to most recent backup
scripts/manage-zed-extension-local.sh rollback
```

Notes:

- Default install target is auto-detected by platform:
  - macOS: `~/Library/Application Support/Zed/extensions/installed`
  - Linux: `~/.local/share/zed/extensions/installed` (fallback `~/.local/share/Zed/extensions/installed`)
- Backups are timestamped under:
  - `<extensions-dir>/.sec4-backups/untrusted/`
- You can force a target path with:
  - `--extensions-dir <path>`
- For live-reload style local development, use:
  - `--mode symlink`

## Binary resolution order

The extension resolves the LSP binary in this order:

1. `lsp.sec4audit-lsp.binary.path` from Zed settings (if provided)
2. `sec4audit-language-server` on `PATH`
3. local workspace fallback paths:
   - `<worktree>/target/debug/sec4audit-language-server`
   - `<worktree>/target/release/sec4audit-language-server`
   - `<worktree>/compiler/sec4-lsp/target/debug/sec4audit-language-server`
   - `<worktree>/compiler/sec4-lsp/target/release/sec4audit-language-server`

If not found, the extension now returns a clear error with searched paths and recovery steps.

## Optional Zed settings override

Use this when you want an explicit binary path:

```json
{
  "lsp": {
    "sec4audit-lsp": {
      "binary": {
        "path": "/absolute/path/to/sec4audit-language-server",
        "arguments": ["--stdio"]
      }
    }
  }
}
```

## Notes

- `grammars.untrusted.rev` is pinned to a commit SHA and should be updated whenever grammar changes are intentionally rolled forward.

## Validation

Use the release checklist before publishing:

```bash
scripts/check-zed-extension-release.sh
```

This checklist includes grammar pin validation and fast smoke checks for the plugin sample project.

For full operator-flow validation (install -> smoke -> rollback in a clean temp profile), run:

```bash
scripts/run-zed-extension-operator-smoke.sh
```

For deterministic local release bundle staging, run:

```bash
scripts/stage-zed-extension-bundle.sh --clean
```

This writes staged bundle content plus `build/zed-extension-bundle/bundle-manifest.json`.
