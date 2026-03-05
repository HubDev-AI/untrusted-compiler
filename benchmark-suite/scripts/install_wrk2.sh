#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dest /abs/path/to/wrk2] [--ref <git-ref>] [--force]

Builds wrk2 from source and installs it as an executable binary.
Default install path: benchmark-suite/bin/wrk2

Environment overrides:
  BENCH_WRK2_REPO_URL   (default: https://github.com/giltene/wrk2.git)
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
suite_dir="$(cd "$script_dir/.." && pwd)"
default_dest="$suite_dir/bin/wrk2"
dest="$default_dest"
repo_url="${BENCH_WRK2_REPO_URL:-https://github.com/giltene/wrk2.git}"
repo_ref="master"
force="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dest)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      dest="$2"
      shift 2
      ;;
    --dest=*)
      dest="${1#--dest=}"
      shift
      ;;
    --ref)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      repo_ref="$2"
      shift 2
      ;;
    --ref=*)
      repo_ref="${1#--ref=}"
      shift
      ;;
    --force)
      force="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

for cmd in git make cc; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing required build tool: $cmd" >&2
    exit 127
  fi
done

if [ -e "$dest" ] && [ "$force" != "true" ]; then
  if [ -x "$dest" ]; then
    echo "wrk2 already installed: $dest"
    exit 0
  fi
  echo "destination exists and is not executable: $dest (use --force to overwrite)" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

src_dir="$tmp_dir/wrk2-src"
echo "cloning wrk2 source: $repo_url (ref: $repo_ref)"
git clone "$repo_url" "$src_dir" >/dev/null
(
  cd "$src_dir"
  git checkout "$repo_ref" >/dev/null
  make >/dev/null
)

mkdir -p "$(dirname "$dest")"
install -m 0755 "$src_dir/wrk" "$dest"
echo "installed wrk2: $dest"
echo "hint: export BENCH_WRK2_BIN=\"$dest\" for benchmark scripts"
