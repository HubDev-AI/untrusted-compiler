# 1043 M39 Slice: Zed Extension Operator Install Guide

This slice adds deterministic local operator workflows for installing, updating, and rolling back the Zed extension from the repository source.

## What changed

1. Added `scripts/manage-zed-extension-local.sh` with commands:
   - `install`
   - `update`
   - `rollback`
   - `status`
2. Install target path resolution is now deterministic with platform defaults and explicit override support:
   - macOS default: `~/Library/Application Support/Zed/extensions/installed`
   - Linux defaults: `~/.local/share/zed/extensions/installed` (fallback `~/.local/share/Zed/extensions/installed`)
   - override: `--extensions-dir <path>`
3. Install/update now preserve previous local state in timestamped backups under:
   - `<extensions-dir>/.sec4-backups/<extension-id>/`
4. Updated `zed-extension/README.md` with operator command flow and rollback notes.

## Why

Before this slice, local extension setup depended on ad-hoc manual copying and no standard rollback path. This adds one command surface for operators and multi-agent lanes so extension validation is reproducible and recoverable.

## Validation

- `bash -n scripts/manage-zed-extension-local.sh`
- `scripts/manage-zed-extension-local.sh status`
- `scripts/manage-zed-extension-local.sh install --mode symlink`
- `scripts/manage-zed-extension-local.sh rollback`

## Notes

- `install` and `update` share the same deterministic behavior: backup existing install then install current repository state.
- `--mode symlink` is useful for local iteration because source edits are reflected immediately without recopying.
