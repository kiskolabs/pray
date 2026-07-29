#!/usr/bin/env bash
# Orchestrate manual release surfaces.
#
# Language registries default to dry-run. Pass --publish to push.
# Distribution-point publish always needs --root and/or --server.
#
# Usage:
#   scripts/release/all.sh
#   scripts/release/all.sh --publish
#   scripts/release/all.sh --publish --root ./prayers --server URL
#   scripts/release/all.sh --skip-crates --skip-npm --skip-rubygems --root ./prayers
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

MODE="dry-run"
SKIP_CRATES=0
SKIP_NPM=0
SKIP_RUBYGEMS=0
SKIP_DISTRIBUTION=0
DISTRIBUTION_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --publish)
      MODE="publish"
      shift
      ;;
    --dry-run)
      MODE="dry-run"
      shift
      ;;
    --skip-crates)
      SKIP_CRATES=1
      shift
      ;;
    --skip-npm)
      SKIP_NPM=1
      shift
      ;;
    --skip-rubygems)
      SKIP_RUBYGEMS=1
      shift
      ;;
    --skip-distribution)
      SKIP_DISTRIBUTION=1
      shift
      ;;
    --root | --server | --signing-key)
      DISTRIBUTION_ARGS+=("$1" "$2")
      shift 2
      ;;
    -h | --help)
      sed -n '2,14p' "$0"
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

REGISTRY_FLAG="--dry-run"
if [[ "${MODE}" == "publish" ]]; then
  REGISTRY_FLAG="--publish"
fi

if [[ "${SKIP_CRATES}" -eq 0 ]]; then
  "${SCRIPT_DIR}/crates.sh" "${REGISTRY_FLAG}"
fi
if [[ "${SKIP_NPM}" -eq 0 ]]; then
  "${SCRIPT_DIR}/npm.sh" "${REGISTRY_FLAG}"
fi
if [[ "${SKIP_RUBYGEMS}" -eq 0 ]]; then
  "${SCRIPT_DIR}/rubygems.sh" "${REGISTRY_FLAG}"
fi

if [[ "${SKIP_DISTRIBUTION}" -eq 0 ]]; then
  if [[ ${#DISTRIBUTION_ARGS[@]} -eq 0 ]]; then
    echo "skipping distribution publish (pass --root and/or --server to enable)"
  else
    DIST_FLAG=()
    if [[ "${MODE}" == "dry-run" ]]; then
      DIST_FLAG=(--dry-run)
    fi
    "${SCRIPT_DIR}/distribution.sh" "${DIST_FLAG[@]}" "${DISTRIBUTION_ARGS[@]}"
  fi
fi

echo "release orchestration finished (${MODE})"
