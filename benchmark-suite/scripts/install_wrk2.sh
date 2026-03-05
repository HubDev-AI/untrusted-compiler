#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dest /abs/path/to/wrk2] [--ref <git-ref>] [--mode auto|build|docker] [--force]

Installs wrk2 for benchmark tooling.
Default install path: benchmark-suite/bin/wrk2

Environment overrides:
  BENCH_WRK2_REPO_URL       (default: https://github.com/giltene/wrk2.git)
  BENCH_WRK2_DOCKER_IMAGE   (default: adysonmaia/wrk2)

Modes:
  auto   Try source build first; if build fails and docker is available, install docker wrapper fallback.
  build  Source-build only.
  docker Install docker wrapper only.
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
suite_dir="$(cd "$script_dir/.." && pwd)"
default_dest="$suite_dir/bin/wrk2"
dest="$default_dest"
repo_url="${BENCH_WRK2_REPO_URL:-https://github.com/giltene/wrk2.git}"
repo_ref="master"
mode="auto"
docker_image="${BENCH_WRK2_DOCKER_IMAGE:-adysonmaia/wrk2}"
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
    --mode)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      mode="$2"
      shift 2
      ;;
    --mode=*)
      mode="${1#--mode=}"
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

case "$mode" in
  auto|build|docker) ;;
  *)
    echo "unsupported mode: $mode (expected auto|build|docker)" >&2
    exit 2
    ;;
esac

if [ -e "$dest" ] && [ "$force" != "true" ]; then
  if [ -x "$dest" ]; then
    echo "wrk2 already installed: $dest"
    exit 0
  fi
  echo "destination exists and is not executable: $dest (use --force to overwrite)" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
build_log="/tmp/wrk2-build-$(date +%s).log"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

install_from_source() {
  for cmd in git make cc; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
      echo "missing required source-build tool: $cmd" >&2
      return 127
    fi
  done

  local src_dir="$tmp_dir/wrk2-src"
  echo "cloning wrk2 source: $repo_url (ref: $repo_ref)"
  if ! git clone "$repo_url" "$src_dir" >/dev/null 2>&1; then
    echo "failed to clone wrk2 source repository: $repo_url" >&2
    return 1
  fi
  if ! (
    cd "$src_dir"
    git checkout "$repo_ref" >/dev/null 2>&1
    make >"$build_log" 2>&1
  ); then
    echo "wrk2 source build failed (log: $build_log)" >&2
    tail -n 40 "$build_log" >&2 || true
    return 1
  fi

  mkdir -p "$(dirname "$dest")"
  install -m 0755 "$src_dir/wrk" "$dest"
  echo "installed source-built wrk2: $dest"
  return 0
}

install_docker_wrapper() {
  if ! command -v docker >/dev/null 2>&1; then
    echo "docker is required for docker wrapper mode but was not found in PATH" >&2
    return 127
  fi

  mkdir -p "$(dirname "$dest")"
  cat >"$dest" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required for wrk2 docker wrapper mode" >&2
  exit 127
fi

image="${BENCH_WRK2_DOCKER_IMAGE:-adysonmaia/wrk2}"
docker_entry="${BENCH_WRK2_DOCKER_ENTRY:-/usr/local/bin/wrk}"
docker_add_host="${BENCH_WRK2_DOCKER_ADD_HOST:-1}"
docker_mount_root="${BENCH_WRK2_DOCKER_MOUNT_ROOT:-$PWD}"

rewrite_url_if_localhost() {
  local value="$1"
  value="${value/http:\/\/127.0.0.1/http:\/\/host.docker.internal}"
  value="${value/https:\/\/127.0.0.1/https:\/\/host.docker.internal}"
  value="${value/http:\/\/localhost/http:\/\/host.docker.internal}"
  value="${value/https:\/\/localhost/https:\/\/host.docker.internal}"
  printf '%s\n' "$value"
}

normalize_existing_path() {
  local candidate="$1"
  if [ -f "$candidate" ]; then
    printf '%s/%s\n' "$(cd "$(dirname "$candidate")" && pwd)" "$(basename "$candidate")"
    return 0
  fi
  if [ -f "$docker_mount_root/$candidate" ]; then
    printf '%s/%s\n' "$(cd "$(dirname "$docker_mount_root/$candidate")" && pwd)" "$(basename "$candidate")"
    return 0
  fi
  return 1
}

mount_dirs=()
has_mount_dir() {
  local probe="$1"
  local existing=""
  for existing in "${mount_dirs[@]:-}"; do
    if [ "$existing" = "$probe" ]; then
      return 0
    fi
  done
  return 1
}
add_mount_dir() {
  local dir="$1"
  [ -d "$dir" ] || return 0
  if ! has_mount_dir "$dir"; then
    mount_dirs+=("$dir")
  fi
}

if [ -d "$docker_mount_root" ]; then
  docker_mount_root="$(cd "$docker_mount_root" && pwd)"
  add_mount_dir "$docker_mount_root"
fi

args=()
raw_args=("$@")
i=0
while [ "$i" -lt "${#raw_args[@]}" ]; do
  arg="${raw_args[$i]}"
  if [ "$arg" = "-s" ] && [ $((i + 1)) -lt "${#raw_args[@]}" ]; then
    script_arg="${raw_args[$((i + 1))]}"
    if script_abs="$(normalize_existing_path "$script_arg" 2>/dev/null)"; then
      add_mount_dir "$(dirname "$script_abs")"
      args+=("-s" "$script_abs")
    else
      args+=("-s" "$script_arg")
    fi
    i=$((i + 2))
    continue
  fi
  case "$arg" in
    http://127.0.0.1*|https://127.0.0.1*|http://localhost*|https://localhost*)
      args+=("$(rewrite_url_if_localhost "$arg")")
      ;;
    *)
      args+=("$arg")
      ;;
  esac
  i=$((i + 1))
done

docker_cmd=(docker run --rm)
if [ "$docker_add_host" = "1" ]; then
  docker_cmd+=(--add-host=host.docker.internal:host-gateway)
fi
for mount_dir in "${mount_dirs[@]:-}"; do
  docker_cmd+=(-v "${mount_dir}:${mount_dir}:ro")
done
if [ -n "${docker_mount_root:-}" ] && [ -d "$docker_mount_root" ]; then
  docker_cmd+=(-w "$docker_mount_root")
fi
while IFS='=' read -r env_name _; do
  case "$env_name" in
    BENCH_*)
      docker_cmd+=(-e "$env_name")
      ;;
  esac
done < <(env)
docker_cmd+=("$image" "$docker_entry")
docker_cmd+=("${args[@]}")

exec "${docker_cmd[@]}"
EOF
  chmod +x "$dest"
  echo "installed wrk2 docker wrapper: $dest"
  echo "wrapper defaults: image=${docker_image} entry=/usr/local/bin/wrk"
  return 0
}

case "$mode" in
  build)
    install_from_source
    ;;
  docker)
    install_docker_wrapper
    ;;
  auto)
    if install_from_source; then
      :
    else
      echo "falling back to docker wrapper install" >&2
      install_docker_wrapper
    fi
    ;;
esac

echo "hint: export BENCH_WRK2_BIN=\"$dest\" for benchmark scripts"
