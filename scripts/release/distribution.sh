#!/usr/bin/env bash
# Package path-owned prayers from the project Prayfile and publish them.
#
# Remote git dependencies are not packed. pray package and pray publish
# select path-owned packages only (RFC 0118).
#
# Usage:
#   scripts/release/distribution.sh --root ./prayers
#   scripts/release/distribution.sh --server https://example.invalid/pray
#   scripts/release/distribution.sh --root ./prayers --server URL --signing-key PATH
#
# Environment:
#   PRAY                 pray binary (default: pray)
#   PRAY_RELEASE_YES=1   skip confirmation prompts
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

ROOT="$(release_repo_root)"
PRAY_BIN="${PRAY:-pray}"
ROOTS=()
SERVERS=()
SIGNING_KEY=""
DRY_RUN=0

usage() {
  sed -n '2,16p' "$0"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --root)
      ROOTS+=("$2")
      shift 2
      ;;
    --server)
      SERVERS+=("$2")
      shift 2
      ;;
    --signing-key)
      SIGNING_KEY="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ ${#ROOTS[@]} -eq 0 && ${#SERVERS[@]} -eq 0 ]]; then
  echo "distribution publish requires at least one --root PATH or --server URL" >&2
  exit 2
fi

release_require_command "${PRAY_BIN}"

cd "${ROOT}"

echo "==> ${PRAY_BIN} package ($(command -v "${PRAY_BIN}"))"
"${PRAY_BIN}" --version 2>/dev/null || true

if [[ "${DRY_RUN}" -eq 1 ]]; then
  echo "==> dry-run: would run ${PRAY_BIN} package"
else
  "${PRAY_BIN}" package
fi

PUBLISH_ARGS=()
if ((${#ROOTS[@]} > 0)); then
  for root in "${ROOTS[@]}"; do
    if [[ "${root}" != /* ]]; then
      root="${ROOT}/${root}"
    fi
    mkdir -p "${root}"
    PUBLISH_ARGS+=(--root "${root}")
  done
fi
if ((${#SERVERS[@]} > 0)); then
  for server in "${SERVERS[@]}"; do
    PUBLISH_ARGS+=(--server "${server}")
  done
fi
if [[ -n "${SIGNING_KEY}" ]]; then
  PUBLISH_ARGS+=(--signing-key "${SIGNING_KEY}")
fi

if [[ "${DRY_RUN}" -eq 1 ]]; then
  echo "==> dry-run: would run ${PRAY_BIN} publish ${PUBLISH_ARGS[*]}"
  exit 0
fi

if ! release_confirm "Publish local prayers to distribution point?"; then
  echo "skipped distribution publish"
  exit 0
fi

echo "==> ${PRAY_BIN} publish ${PUBLISH_ARGS[*]}"
"${PRAY_BIN}" publish "${PUBLISH_ARGS[@]}"

echo "distribution publish finished"
