#!/usr/bin/env bash
# Package local prayer packages and publish them to a distribution point.
#
# Builds a temporary publisher workspace that only declares packages under
# packages/, then runs `pray package` and `pray publish`.
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

PACKAGES_DIR="${ROOT}/packages"
if [[ ! -d "${PACKAGES_DIR}" ]]; then
  echo "missing packages directory: ${PACKAGES_DIR}" >&2
  exit 2
fi

PACKAGE_SPECS=()
while IFS= read -r spec; do
  PACKAGE_SPECS+=("${spec}")
done < <(find "${PACKAGES_DIR}" -mindepth 2 -maxdepth 2 -type f -name '*.prayspec' | sort)
if [[ ${#PACKAGE_SPECS[@]} -eq 0 ]]; then
  echo "no *.prayspec files found under ${PACKAGES_DIR}" >&2
  exit 2
fi

PUBLISHER="$(mktemp -d "${TMPDIR:-/tmp}/pray-release-publisher.XXXXXX")"
cleanup() {
  rm -rf "${PUBLISHER}"
}
trap cleanup EXIT

ln -s "${PACKAGES_DIR}" "${PUBLISHER}/packages"

{
  echo 'prayfile "1"'
  echo
  for spec in "${PACKAGE_SPECS[@]}"; do
    package_dir="$(dirname "${spec}")"
    relative="${package_dir#"${PACKAGES_DIR}"/}"
    name="$(basename "${relative}")"
    echo "pray \"${name}\", path: \"packages/${relative}\""
  done
} >"${PUBLISHER}/Prayfile"

echo "publisher workspace: ${PUBLISHER}"
echo "packages:"
for spec in "${PACKAGE_SPECS[@]}"; do
  echo "  - ${spec#"${ROOT}/"}"
done

cd "${PUBLISHER}"

echo "==> ${PRAY_BIN} package"
"${PRAY_BIN}" package

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

if ! release_confirm "Publish local packages to distribution point?"; then
  echo "skipped distribution publish"
  exit 0
fi

echo "==> ${PRAY_BIN} publish ${PUBLISH_ARGS[*]}"
"${PRAY_BIN}" publish "${PUBLISH_ARGS[@]}"

echo "distribution publish finished"
